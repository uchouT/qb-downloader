use std::{
    collections::HashMap,
    path::Path,
    sync::{
        LazyLock, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard,
        mpsc::{SendError, Sender},
    },
};

use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{
    errors::{ContextedResult, TaskError},
    qb,
};
const FETCHED_STATE: [&str; 4] = ["stoppedUP", "pausedUP", "stoppedDL", "pausedDL"];
type FetchingMap = HashMap<String, MetadataFetchingTask>;
static METADATA_FETCHING_MAP: LazyLock<RwLock<FetchingMap>> =
    LazyLock::new(|| RwLock::new(FetchingMap::new()));

type FetchHandle = JoinHandle<Result<(), TaskError>>;
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
        self.cancel_sender.set(tx).expect("Never");
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
    fetching_map_mut().insert(hash, medata_fetching_task);
}

pub(super) fn fetching_task_handle(hash: &str) -> Option<FetchHandle> {
    fetching_map_mut().get_mut(hash)?.get_handle()
}

/// Cancel a fetching task by hash
pub(super) fn cancel(hash: &str) -> Result<(), FetchingError> {
    if let Some(fetching_task) = fetching_map_mut().remove(hash) {
        fetching_task
            .cancel_sender
            .get()
            .expect("tx doesn't initialize")
            .send(())?;
    }

    Ok(())
}

/// Export torrent file after fetching metadata
pub(super) async fn export(hash: impl AsRef<str>, path: impl AsRef<Path>) -> Result<(), TaskError> {
    let (hash, path) = (hash.as_ref(), path.as_ref());
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    fetching_map().get(hash).unwrap().set_cancel_sender(tx);
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
    Closed,
}

impl<T> From<SendError<T>> for FetchingError {
    fn from(_: SendError<T>) -> Self {
        Self::Closed
    }
}
