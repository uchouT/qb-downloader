//! This module handle task process

use crate::{
    errors::{AppError, ContextedResult, TargetContextedResult, format_error_chain},
    qb, request,
    task::{
        self, RuntimeTaskError, Status, TaskMap, TaskValue,
        error::{RuntimeTaskErrorKind, TaskError},
        launch, task_map, task_map_mut,
    },
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

pub async fn run(mut shutdown_rx: broadcast::Receiver<()>) -> Result<(), AppError> {
    request::init().await;
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
                error!("Failed to process task list\n{e:?}");
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
async fn process_task_list() -> Result<(), AppError> {
    if task_map().is_empty() {
        return Ok(());
    }
    update_task()
        .await
        .convert_then_add_context("Failed to update task")?;
    let futures: Vec<_> = {
        let task_map = task_map();
        task_map
            .values()
            .map(|task| {
                let task = task.clone();
                async move {
                    if let Err(e) = process_task(task.clone()).await {
                        handle_task_error(task, e);
                    }
                }
            })
            .collect()
    };

    join_all(futures).await;
    task::save()
        .await
        .convert_then_add_context("Failed to save task list")?;
    Ok(())
}

/// Called when a task encounters a runtime error
fn handle_task_error(task: Arc<TaskValue>, e: RuntimeTaskError) {
    log::error!(
        "Task: {} encountered an error\n{}",
        &task.name,
        format_error_chain(&e)
    );
    let mut state = task.state_mut();
    state.status = Status::Error;
    task.set_error_info(e);
}

/// process single task
/// # Error
/// - may return [`RuntimeTaskErrorKind`]
async fn process_task(task: Arc<TaskValue>) -> Result<(), RuntimeTaskError> {
    let (status, is_seeding) = {
        let state = task.state();
        (state.status, state.is_seeding)
    };

    match status {
        Status::OnTask => {
            task.run_check().await.map_err(|e| {
                RuntimeTaskError::from_kind(RuntimeTaskErrorKind::RuntimeUpload, Some(e))
            })?;
            Ok(())
        }
        Status::Downloaded => {
            task.run_interval().await.map_err(|e| {
                RuntimeTaskError::from_kind(RuntimeTaskErrorKind::LaunchUpload, Some(e))
            })?;
            Ok(())
        }
        Status::Finished => {
            if is_seeding {
                Ok(())
            } else {
                add_next_part(task.clone()).await.map_err(|e| {
                    RuntimeTaskError::from_kind(RuntimeTaskErrorKind::AddNextPart, Some(e))
                })?;
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

enum TorrentState {
    // torrent is seeding, which means torrent finished downloading
    Seeding,
    Error,
    // torrent finished both downloading and seeding
    FinishedSeeding,
    Downloading,
}

fn classify_torrent_state(state: String) -> TorrentState {
    use TorrentState::*;
    if SEEDING.contains(&state.as_str()) {
        Seeding
    } else if ERROR.contains(&state.as_str()) {
        Error
    } else if FINISHED_SEEDING.contains(&state.as_str()) {
        FinishedSeeding
    } else {
        Downloading
    }
}

/// update task status, task map is not empty
/// # Error
/// - may return [`TaskError::Qb`] is get torrents status failed
async fn update_task() -> Result<(), TaskError> {
    let torrent_infos = qb::get_torrent_info()
        .await
        .add_context("Failed to get torrent infos")?;

    let mut task_map = task_map_mut();
    let mut new_task_map = TaskMap::new();

    for info in torrent_infos {
        let task = task_map.remove(&info.hash);

        if let Some(task) = task {
            let mut state = task.state_mut();
            let current_status = state.status;
            let current_seeding = state.is_seeding;

            // check if downloading has completed
            if let Status::Downloading = current_status {
                state.progress = info.progress;
                let torrent_staus = classify_torrent_state(info.state);

                use TorrentState::*;
                match torrent_staus {
                    Seeding => {
                        state.status = Status::Downloaded;
                        state.is_seeding = true;
                    }
                    FinishedSeeding => {
                        state.status = Status::Downloaded;
                        state.is_seeding = false;
                    }
                    Error => {
                        state.status = Status::Error;
                        task.set_error_info(RuntimeTaskError::from_kind(
                            RuntimeTaskErrorKind::Download,
                            None,
                        ));
                    }
                    Downloading => {}
                }
            }
            // check if seeding has finished
            else if current_seeding && FINISHED_SEEDING.contains(&info.state.as_str()) {
                state.is_seeding = false;
            }
            drop(state);
            new_task_map.insert(info.hash, task);
        }
    }

    // remaining tasks in old map are probably deleted in qbittorrent
    std::mem::take(&mut *task_map)
        .into_iter()
        .for_each(|(hash, task)| {
            let status = task.state().status;
            if status != Status::Done && status != Status::Error {
                error!("Task: {} not found in qbittorrent", &task.name);
                task.state_mut().status = Status::Error;
                task.set_error_info(RuntimeTaskError::from_kind(
                    RuntimeTaskErrorKind::TorrentNotFound,
                    None,
                ));
            }
            new_task_map.insert(hash, task);
        });
    *task_map = new_task_map;
    Ok(())
}

/// Add the next part of the task, may get task state write lock
/// # Error
/// may return [`RuntimeTaskError::AddNextPart`]
pub(super) async fn add_next_part(task: Arc<TaskValue>) -> Result<(), TaskError> {
    let hash = &task.hash;
    let (current_part_num, total_parts) = { (task.state().current_part_num, task.total_part_num) };
    qb::delete(hash, true)
        .await
        .add_context("Failed to delete old part")?;

    if current_part_num == total_parts - 1 {
        task.state_mut().status = Status::Done;
        info!("Task: {} completed", &task.name);
        return Ok(());
    }
    let new_part_num = current_part_num + 1;
    add_part(new_part_num, task).await
}

/// Add torrent from cached, launch given index part
pub(super) async fn add_part(index: usize, task: Arc<TaskValue>) -> Result<(), TaskError> {
    qb::add_by_file(
        &task.torrent_path,
        &task.save_path,
        task.seeding_time_limit,
        task.ratio_limit,
    )
    .await
    .add_context("Failed to add to qbittorrent")?;
    sleep(Duration::from_millis(500)).await;
    launch(index, &task.hash, task.clone()).await?;
    info!("Added part {} for task: {}", index + 1, &task.name);
    Ok(())
}

async fn shutdown() -> Result<(), crate::errors::CommonError> {
    task::save().await?;
    Ok(())
}
