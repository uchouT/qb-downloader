// TODO: needs tesk
use crate::task::handle::{add_next_part, add_part};
use super::*;

/// Skip a task from [`Status::Error`] state
pub(super) async fn skip_task(
    task: Arc<TaskValue>,
    kind: RuntimeTaskErrorKind,
) -> Result<(), TaskError> {
    use RuntimeTaskErrorKind::*;
    match kind {
        // re-add torrent and launch from current_part_num
        TorrentNotFound => handle_torrent_not_found(task).await,
        // ignore the runtime upload mistake, directly marked upload success
        RuntimeUpload => {
            task.state_mut().status = Status::Finished;
            Ok(())
        }
        _ => panic!("other kinds are not skipable"),
    }
}

/// user may handle the error themselves, resume the task from last it occurred
pub(super) async fn resume_from_error(
    task: Arc<TaskValue>,
    kind: RuntimeTaskErrorKind,
) -> Result<(), TaskError> {
    use RuntimeTaskErrorKind::*;
    match kind {
        // re-launch upload
        LaunchUpload | RuntimeUpload => task.run_interval().await,
        // re-add next part
        AddNextPart => add_next_part(task).await,
        // try to set category, and other configuration, then mark state [`Status::Downloading`],
        // user needs to fix qbittorrent download issue, probably network, storage, etc.
        Download => {
            let _ = qb::delete(&task.hash, false)
                .await;
            let current_part_num = task.state().current_part_num;
            add_part(current_part_num, task).await
        }
        TorrentNotFound => handle_torrent_not_found(task).await,
    }
}

async fn handle_torrent_not_found(task: Arc<TaskValue>) -> Result<(), TaskError> {
    let current_part_num = task.state().current_part_num;
    add_part(current_part_num, task).await
}
