use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, LazyLock, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard, mpsc::Sender},
};

use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{
    errors::{ContextedResult, TaskError},
    qb,
};
const FETCHED_STATE: [&str; 4] = ["stoppedUP", "pausedUP", "stoppedDL", "pausedDL"];
type FetchingMap = HashMap<String, FetchingTaskItem>;
type FetchHandle = JoinHandle<Result<(), TaskError>>;

#[derive(Clone)]
struct FetchingTaskItem(Arc<RwLock<MetadataFetchingTask>>);

impl FetchingTaskItem {
    fn read(&self) -> RwLockReadGuard<'_, MetadataFetchingTask> {
        self.0.read().unwrap()
    }
    fn write(&self) -> RwLockWriteGuard<'_, MetadataFetchingTask> {
        self.0.write().unwrap()
    }
    fn from_fetching_task(task: MetadataFetchingTask) -> Self {
        Self(Arc::from(RwLock::from(task)))
    }
}

static METADATA_FETCHING_MAP: LazyLock<RwLock<FetchingMap>> =
    LazyLock::new(|| RwLock::new(FetchingMap::new()));

pub(super) struct MetadataFetchingTask {
    cancel_sender: OnceLock<Sender<()>>,
    handle: Option<FetchHandle>,
}

impl MetadataFetchingTask {
    fn from_handle(handle: FetchHandle) -> Self {
        Self {
            cancel_sender: OnceLock::new(),
            handle: Some(handle),
        }
    }

    fn set_cancel_sender(&self, tx: Sender<()>) {
        self.cancel_sender.set(tx).unwrap();
    }

    const fn get_handle(&mut self) -> Option<FetchHandle> {
        self.handle.take()
    }
}

fn fetching_map() -> RwLockReadGuard<'static, FetchingMap> {
    METADATA_FETCHING_MAP.read().unwrap()
}

fn fetching_map_mut() -> RwLockWriteGuard<'static, FetchingMap> {
    METADATA_FETCHING_MAP.write().unwrap()
}

/// Put a fetching task into background
pub(super) fn insert_fetching_task(hash: String, handle: FetchHandle) {
    let medata_fetching_task = MetadataFetchingTask::from_handle(handle);
    fetching_map_mut().insert(
        hash,
        FetchingTaskItem::from_fetching_task(medata_fetching_task),
    );
}

pub(super) fn fetching_task_handle(hash: &str) -> Option<FetchHandle> {
    fetching_map().get(hash)?.write().get_handle()
}

pub(super) fn finished_fetching_task(hash: &str) {
    fetching_map_mut().remove(hash);
}

/// Cancel a fetching task by hash
/// Send stop signal and remove fetching task from [`METADATA_FETCHING_MAP`]
pub(super) fn cancel(hash: &str) -> Result<(), FetchingError> {
    if let Some(fetching_task) = fetching_map_mut().remove(hash) {
        fetching_task
            .read()
            .cancel_sender
            .get()
            .expect("tx doesn't initialize")
            .send(())
            .map_err(|_| FetchingError::Cancelled)?;
        Ok(())
    } else {
        Err(FetchingError::Finished)
    }
}

/// Export torrent file after fetching metadata
/// Preconditions
/// - call only once per fetching task
/// - the fetching task has been added to [`METADATA_FETCHING_MAP`]
pub(super) async fn export(hash: impl AsRef<str>, path: impl AsRef<Path>) -> Result<(), TaskError> {
    let (hash, path) = (hash.as_ref(), path.as_ref());
    let (tx, rx) = std::sync::mpsc::channel::<()>();

    fetching_map()
        .get(hash)
        .expect("can't find fetching task")
        .write()
        .set_cancel_sender(tx);

    // wait for the torrent to fetch meta data
    loop {
        // recv cancel signal
        if rx.try_recv().is_ok() {
            return Err(TaskError::Abort);
        }

        let state = qb::get_state(hash)
            .await
            .add_context("Failed to get qBittorrent state")?;
        if FETCHED_STATE.contains(&state.as_str()) {
            drop(rx);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    qb::export(hash, path)
        .await
        .add_context("Failed to export torrent file")?;
    Ok(())
}

#[derive(Debug, Error)]
pub(super) enum FetchingError {
    #[error("Failed to send cancel signal")]
    Cancelled,

    #[error("Failed to find fetching task, perhaps finished")]
    Finished,
}
