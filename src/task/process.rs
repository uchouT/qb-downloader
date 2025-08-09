use crate::{
    Entity, format_error_chain, qb,
    task::{Status, Task, TaskItem, error::TaskError, launch},
};
use futures::future::join_all;
use log::{error, info, warn};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::time::{Duration, sleep};

const SEEDING: [&str; 5] = [
    "uploading",
    "stalledUP",
    "queuedUP",
    "checkingUP",
    "forcedUP",
];
const FINISHED_SEEDING: [&str; 2] = ["stoppedUP", "pausedUP"];
const ERROR: [&str; 2] = ["error", "missingFiles"];

pub async fn run(running: Arc<AtomicBool>) -> Result<(), TaskError> {
    qb::login().await;
    let mut first_login = true;

    while running.load(Ordering::Relaxed) {
        while qb::is_logined().await && running.load(Ordering::Relaxed) {
            if first_login {
                first_login = false;
                info!("qBittorrent login successful");
            }
            if let Err(e) = process_task_list().await {
                error!("Failed to process task list: {}", format_error_chain(e));
            }
            sleep(Duration::from_secs(5)).await;
        }
        // qBittorrent not logined, retry after 10 secs
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}

/// process all tasks
async fn process_task_list() -> Result<(), TaskError> {
    update_task().await.inspect_err(|_| {
        error!("Failed to update task");
    })?;

    let mut process_task_list = Vec::new();
    Task::read(|task_list| {
        if task_list.is_empty() {
            return;
        }
        task_list.iter().for_each(|(hash, task)| {
            process_task_list.push((hash.clone(), task.clone()));
        });
    })
    .await;

    if process_task_list.is_empty() {
        return Ok(());
    }
    let futures = process_task_list
        .into_iter()
        .map(|(hash, task)| process_task(task, hash));
    let _results = join_all(futures).await;
    if let Err(e) = Task::save().await {
        error!("Failed to save task list: {}", format_error_chain(&e));
    }
    Ok(())
}

/// process single task
async fn process_task(task: Arc<TaskItem>, hash: String) -> Result<(), TaskError> {
    let (status, is_seeding) = {
        let read_guard = task.0.read().await;
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
            task.0.write().await.status = Status::Error;
            error!(
                "Failed to add next part for task: {}",
                task.0.read().await.name
            )
        }
    }
    Ok(())
}

/// update task status
async fn update_task() -> Result<(), TaskError> {
    let torrent_infos = qb::get_torrent_info().await?;
    for info in torrent_infos {
        let task = Task::read(|task_list| task_list.get(&info.hash).cloned()).await;
        if let Some(task) = task {
            let (current_status, current_seeding) = {
                let task_read = task.0.read().await;
                (task_read.status, task_read.is_seeding)
            };

            if current_status == Status::Downloading {
                let progress = info.progress;
                let state = info.state;

                // torrent is seeding
                if SEEDING.contains(&state.as_str()) {
                    let mut task_write_guard = task.0.write().await;
                    task_write_guard.progress = progress;
                    task_write_guard.status = Status::Downloaded;
                    task_write_guard.is_seeding = true;
                    continue;
                }

                // torrent finished both downloading and seeding
                if FINISHED_SEEDING.contains(&state.as_str()) {
                    let mut task_write_guard = task.0.write().await;
                    task_write_guard.progress = progress;
                    task_write_guard.status = Status::Downloaded;
                    task_write_guard.is_seeding = false;
                    continue;
                }

                if ERROR.contains(&state.as_str()) {
                    let mut task_write_guard = task.0.write().await;
                    task_write_guard.progress = progress;
                    task_write_guard.status = Status::Error;
                    continue;
                }
                continue;
            }
            if FINISHED_SEEDING.contains(&info.state.as_str()) && current_seeding {
                let mut task_write_guard = task.0.write().await;
                task_write_guard.is_seeding = false;
            }
        } else {
            warn!("No task found for torrent {}", info.hash);
        }
    }
    Ok(())
}

/// Add the next part of the task
async fn add_next_part(task: Arc<TaskItem>, hash: &str) -> Result<(), TaskError> {
    let (current_part_num, total_parts) = {
        let read_guard = task.0.read().await;
        (read_guard.current_part_num, read_guard.total_part_num)
    };
    qb::delete(hash, true).await.inspect_err(|e| {
        error!("Failed to delete {hash} \n{}", format_error_chain(e));
    })?;

    if current_part_num < total_parts - 1 {
        let name = {
            let read_guard = task.0.read().await;
            let name = &read_guard.name;
            qb::add_by_file(
                &read_guard.torrent_path,
                &read_guard.save_path,
                read_guard.seeding_time_limit,
                read_guard.ratio_limit,
            )
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to add next part for task: {name} \n {}",
                    format_error_chain(e)
                );
            })?;
            name.clone()
        };
        let new_part_num = current_part_num + 1;
        sleep(Duration::from_millis(500)).await;
        launch(new_part_num, hash, &mut *task.0.write().await).await?;
        info!("Added next part {} for task: {name}", new_part_num + 1);
    } else {
        task.0.write().await.status = Status::Done;
        info!("Task: {} completed", task.0.read().await.name);
    }

    Ok(())
}
