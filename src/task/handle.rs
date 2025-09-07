//! This module handle task process

use crate::{
    Error, format_error_chain, qb,
    task::{self, Status, TaskValue, error::TaskError, launch, task_map},
};
use futures_util::{FutureExt, future::join_all, select};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::{
    sync::broadcast,
    time::{Duration, interval, sleep},
};

const SEEDING: [&str; 5] = [
    "uploading",
    "stalledUP",
    "queuedUP",
    "checkingUP",
    "forcedUP",
];
const FINISHED_SEEDING: [&str; 2] = ["stoppedUP", "pausedUP"];
const ERROR: [&str; 2] = ["error", "missingFiles"];

pub async fn run(mut shutdown_rx: broadcast::Receiver<()>) -> Result<(), Error> {
    qb::init();
    qb::login().await;
    let mut task_interval = interval(Duration::from_secs(5));
    loop {
        select! {
            _ = shutdown_rx.recv().fuse() => {
                break;
            }

            _ = task_interval.tick().fuse() => {
                if !qb::is_logined() {
                    continue;
                }
                 if let Err(e) = process_task_list().await {
                error!("Failed to process task list: {e}");
                }
            }
        }
    }
    info!("Task handler starting shutdown...");
    select! {
        result = shutdown().fuse() => {
            if let Err(e) = result {
                error!("Failed to shutdown task handler: {e}");
            }
        }
        _ = sleep(Duration::from_secs(5)).fuse() => {
            warn!("Task handler shutdown timed out, continuing...");
        }
    }
    Ok(())
}

/// process all tasks
async fn process_task_list() -> Result<(), TaskError> {
    update_task().await.inspect_err(|_| {
        error!("Failed to update task");
    })?;

    let mut process_task_list = Vec::new();
    {
        let task_map = task_map();
        if !task_map.is_empty() {
            task_map.iter().for_each(|(hash, task)| {
                process_task_list.push((hash.clone(), task.clone()));
            });
        }
    }

    if process_task_list.is_empty() {
        return Ok(());
    }
    let futures = process_task_list
        .into_iter()
        .map(|(hash, task)| process_task(task, hash));
    let _results = join_all(futures).await;
    if let Err(e) = task::save().await {
        error!("Failed to save task list: {}", format_error_chain(&e));
    }
    Ok(())
}

/// process single task
async fn process_task(task: Arc<TaskValue>, hash: String) -> Result<(), TaskError> {
    let (status, is_seeding) = {
        let read_guard = task.state();
        (read_guard.status, read_guard.is_seeding)
    };
    if status == Status::Downloading
        || status == Status::Paused
        || status == Status::Done
        || status == Status::Error
    {
        return Ok(());
    }
    if status == Status::OnTask {
        tokio::spawn(task.run_check());
        return Ok(());
    }
    if status == Status::Downloaded {
        tokio::spawn(task.run_interval());
        return Ok(());
    }
    if status == Status::Finished {
        if is_seeding {
            return Ok(());
        }
        if add_next_part(task.clone(), &hash).await.is_err() {
            task.state_mut().status = Status::Error;
            error!("Failed to add next part for task: {}", &task.name)
        }
    }
    Ok(())
}

/// update task status
async fn update_task() -> Result<(), TaskError> {
    let torrent_infos = qb::get_torrent_info().await?;
    let task_list = task_map();
    for info in torrent_infos {
        let task = task_list.get(&info.hash).cloned();
        if let Some(task) = task {
            let (current_status, current_seeding) = {
                let state = task.state();
                (state.status, state.is_seeding)
            };

            if current_status == Status::Downloading {
                let progress = info.progress;
                let state = info.state;

                // torrent is seeding
                if SEEDING.contains(&state.as_str()) {
                    let mut state = task.state_mut();
                    state.progress = progress;
                    state.status = Status::Downloaded;
                    state.is_seeding = true;
                    continue;
                }

                // torrent finished both downloading and seeding
                if FINISHED_SEEDING.contains(&state.as_str()) {
                    let mut state = task.state_mut();
                    state.progress = progress;
                    state.status = Status::Downloaded;
                    state.is_seeding = false;
                    continue;
                }

                if ERROR.contains(&state.as_str()) {
                    let mut state = task.state_mut();
                    state.progress = progress;
                    state.status = Status::Error;
                    continue;
                }
                let mut state = task.state_mut();
                state.progress = progress;
                continue;
            }

            // check if seeding finished
            if current_seeding && FINISHED_SEEDING.contains(&info.state.as_str()) {
                task.state_mut().is_seeding = false;
            }
        }
    }
    Ok(())
}

/// Add the next part of the task
async fn add_next_part(task: Arc<TaskValue>, hash: &str) -> Result<(), TaskError> {
    let (current_part_num, total_parts) = { (task.state().current_part_num, task.total_part_num) };
    qb::delete(hash, true).await.inspect_err(|e| {
        error!("Failed to delete {hash} \n{e}");
    })?;

    if current_part_num == total_parts - 1 {
        task.state_mut().status = Status::Done;
        info!("Task: {} completed", &task.name);
        return Ok(());
    }

    qb::add_by_file(
        &task.torrent_path,
        &task.save_path,
        task.seeding_time_limit,
        task.ratio_limit,
    )
    .await
    .inspect_err(|e| {
        error!(
            "Failed to add next part for task: {} \n {}",
            &task.name,
            format_error_chain(e)
        );
    })?;

    let new_part_num = current_part_num + 1;
    launch(new_part_num, hash, task.clone().as_ref()).await?;
    info!("Added part {} for task: {}", new_part_num + 1, &task.name);
    Ok(())
}

async fn shutdown() -> Result<(), crate::error::CommonError> {
    task::save().await?;
    Ok(())
}
