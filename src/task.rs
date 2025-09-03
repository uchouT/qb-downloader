//! qb-downloader task manager
pub mod error;
pub mod handle;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use directories_next::BaseDirs;
use futures_util::future::{join, join_all};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    Entity, bencode,
    error::{CommonError, TaskError},
    format_error_chain,
    qb::{self, error::QbErrorKind},
    task::error::TaskErrorKind,
    upload::Uploader,
};

const TASK_FILE_NAME: &str = "tasks.json";
const TORRENT_DIR_NAME: &str = "torrents";
static ADD_TORRENT_LOCK: Mutex<()> = Mutex::const_new(());
pub static TASK_LIST: OnceLock<RwLock<Task>> = OnceLock::new();
static TORRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub type TaskMap = BTreeMap<String, Arc<TaskItem>>;
#[derive(Debug)]
pub struct Task {
    pub filepath: PathBuf,
    pub value: TaskMap,
}

#[derive(Debug)]
pub struct TaskItem(pub RwLock<TaskValue>);
/// task value
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskValue {
    pub hash: String,
    pub name: String,

    /// each task must contains a root dir,
    /// single file task is not support
    pub root_dir: String,
    pub status: Status,
    pub save_path: String,
    pub upload_path: String,
    pub uploader: Uploader,
    pub total_part_num: usize,
    pub current_part_num: usize,
    pub task_order: Vec<Vec<usize>>,

    /// total file count, which is used to set not download.
    pub file_num: usize,
    pub torrent_path: PathBuf,
    pub is_seeding: bool,
    pub max_size: i64,
    pub seeding_time_limit: i32,
    pub ratio_limit: f64,
    pub progress: f64,
}

/// task status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Status {
    /// on interval task
    OnTask,

    /// the part task finished incluing interval task
    Finished,

    /// torrent downloaded, to run interval task
    Downloaded,

    /// torrent is still downloading
    Downloading,

    /// the entire task finished, including all parts
    Done,

    Error,
    Paused,
}

impl TaskItem {
    /// Launch the interval task
    pub async fn run_interval(self: Arc<Self>) -> Result<(), TaskError> {
        let mut guard = self.0.write().await;
        guard.status = Status::OnTask;
        info!("Running interval task for: {}", &guard.name);
        let uploader = guard.uploader;
        uploader.upload(&mut guard).await.inspect_err(|e| {
            error!("Failed to run interval task for {}: {e}", &guard.name);
        })
    }

    /// Check if the upload is complete
    pub async fn run_check(self: Arc<Self>) -> Result<(), TaskError> {
        let mut guard = self.0.write().await;
        let uploader = guard.uploader;
        if uploader.check(&mut guard).await.inspect_err(|e| {
            error!("Error occurred while uploading {}: {e}", &guard.name);
        })? {
            guard.status = Status::Finished;
            info!("Upload completed for task: {}", &guard.name);
        }
        Ok(())
    }
}

impl Entity for Task {
    fn new(path: Option<PathBuf>) -> Self {
        let filepath = if let Some(filepath) = path {
            filepath
        } else {
            BaseDirs::new()
                .expect("Failed to get data dir")
                .data_dir()
                .join("qb-downloader")
                .join(TASK_FILE_NAME)
        };
        Task {
            filepath,
            value: BTreeMap::new(),
        }
    }

    fn init(path: Option<PathBuf>) -> Result<(), CommonError> {
        let mut task_list = Self::new(path);
        Self::load(&mut task_list)?;
        debug!("Task list loaded from: {}", &task_list.filepath.display());
        debug!("Task list content: {:?}", &task_list.value);
        TASK_LIST
            .set(RwLock::new(task_list))
            .expect("failed to set task list");
        TORRENT_DIR
            .set(
                BaseDirs::new()
                    .expect("Failed to get data dir")
                    .data_dir()
                    .join("qb-downloader")
                    .join(TORRENT_DIR_NAME),
            )
            .expect("Failed to set torrent directory");
        std::fs::create_dir_all(TORRENT_DIR.get().unwrap())?;
        info!("Task list initialized.");
        Ok(())
    }

    async fn get() -> RwLockReadGuard<'static, Self::LockedValue> {
        TASK_LIST
            .get()
            .expect("task list not initialized")
            .read()
            .await
    }
    async fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue> {
        TASK_LIST
            .get()
            .expect("task list not initialized")
            .write()
            .await
    }
}

pub async fn start(hash: &str) -> Result<(), TaskError> {
    qb::start(hash).await?;

    {
        let task_map = Task::get().await;
        if let Some(task) = task_map.value.get(hash) {
            task.0.write().await.status = Status::Downloading;
        } else {
            let msg = format!("Task not found for hash: {hash}");
            error!("{msg}");
            return Err(TaskError {
                kind: error::TaskErrorKind::Other(msg),
            });
        }
        Task::save_entity(&*task_map).await?;
    }

    info!("Task started for hash: {hash}");
    Ok(())
}

pub async fn stop(hash: &str) -> Result<(), TaskError> {
    qb::stop(hash).await?;

    {
        let task_map = Task::get().await;
        if let Some(task) = task_map.value.get(hash) {
            task.0.write().await.status = Status::Paused;
        } else {
            let msg = format!("Task not found for hash: {hash}");
            error!("{msg}");
            return Err(TaskError {
                kind: error::TaskErrorKind::Other(msg),
            });
        }
        Task::save_entity(&*task_map).await?;
    }

    info!("Task stopped for hash: {hash}");
    Ok(())
}

/// clean the cached torrent file according to hash
pub async fn clean(hash: &str) -> Result<(), TaskError> {
    let path = get_torrent_path(hash);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).await.map_err(|e| {
        error!("Failed to clean waited torrent file: {hash}");
        CommonError::from(e)
    })?;
    Ok(())
}

/// Delete task, both qBittorrent task and cached torrent file.
/// If `added` is true, the task will be removed from the task list.
pub async fn delete(hash: &str, added: bool) -> Result<(), TaskError> {
    let (qb_delete_res, file_clean_res) = join(qb::delete(hash, true), clean(hash)).await;
    if let Err(e) = qb_delete_res {
        error!(
            "Failed to delete torrent in qBittorrent: {}",
            format_error_chain(e)
        );
    }
    if let Err(e) = file_clean_res {
        error!("Failed to clean torrent file: {}", format_error_chain(e));
    }
    if added {
        Task::write(|task_list| task_list.remove(hash)).await;
        if let Err(e) = Task::save().await {
            error!("Failed to save task list: {e}");
        }
    }
    info!("Task deleted for hash: {hash}");
    Ok(())
}

/// Add a new torrent, and return the torrent's hash
/// # Parameters
/// - `is_file`: Whether the torrent is a file or a URL
/// - `file`: The file data
/// - `url`: The filename of the torrent when is_file, otherwise the URL of the torrent
/// - `save_path`: The path to save the download data
pub async fn add_torrent(
    file: Option<&[u8]>,
    url: &str,
    save_path: &str,
) -> Result<String, TaskError> {
    let hash = {
        if let Some(file) = file {
            qb::add_by_bytes(url, save_path, file).await?;
            let hash = bencode::get_hash(file)?;
            let path = get_torrent_path(&hash);
            fs::write(path, file).await.map_err(CommonError::from)?;
            hash
        } else {
            let _lock = ADD_TORRENT_LOCK.lock().await;
            let hash = {
                if let Some(hash) = qb::try_parse_hash(url) {
                    qb::add_by_url(url, save_path).await?;
                    hash
                } else {
                    qb::add_by_url(url, save_path).await?;
                    qb::get_hash().await?
                }
            };
            let path = get_torrent_path(&hash);
            qb::export(&hash, &path).await.map_err(|e| {
                if let QbErrorKind::NoNewTorrents = e.kind {
                    TaskError {
                        kind: TaskErrorKind::Abort,
                    }
                } else {
                    TaskError::from(e)
                }
            })?;
            hash
        }
    };

    qb::add_tag(hash.as_str(), qb::Tag::Waited).await?;
    Ok(hash)
}

pub fn get_torrent_path(hash: &str) -> PathBuf {
    TORRENT_DIR.get().unwrap().join(format!("{hash}.torrent"))
}

/// add task from [`TaskReq`]
pub async fn add(
    hash: String,
    name: String,
    save_path: String,
    upload_path: String,
    uploader: Uploader,
    selected_file_index: Option<Vec<usize>>,
    max_size: i64,
    ratio_limit: f64,
    seeding_time_limit: i32,
) -> Result<(), TaskError> {
    let torrent_path = get_torrent_path(&hash);
    let (root_dir, file_num, task_order) = {
        let value = bencode::get_value(&torrent_path).await?;
        let (root_dir, torrent_lengths_list) = bencode::parse_torrent(&value)?;
        let file_num = torrent_lengths_list.len();
        let task_order = get_task_order(
            &torrent_lengths_list,
            max_size,
            selected_file_index.as_deref(),
        )?;
        (root_dir, file_num, task_order)
    };
    let mut task_value = TaskValue {
        hash: hash.clone(),
        name,
        root_dir,
        status: Status::Paused,
        save_path,
        upload_path,
        uploader,
        total_part_num: task_order.len(),
        current_part_num: 0,
        task_order,
        file_num,
        torrent_path,
        is_seeding: false,
        max_size,
        seeding_time_limit,
        ratio_limit,
        progress: 0.0,
    };
    qb::set_share_limit(&hash, ratio_limit, seeding_time_limit).await?;
    launch(0, &hash, &mut task_value).await?;
    info!("Task added: {}", &hash);
    Task::write(|task_list| task_list.insert(hash, Arc::new(TaskItem(RwLock::new(task_value)))))
        .await;
    Task::save().await?;
    Ok(())
}

/// launch a task
pub async fn launch(index: usize, hash: &str, task: &mut TaskValue) -> Result<(), TaskError> {
    task.current_part_num = index;
    qb::set_not_download(hash, task.file_num).await?;
    qb::set_prio(hash, 1, task.task_order.get(index).unwrap()).await?;
    qb::start(hash).await?;
    qb::remove_tag(hash, qb::Tag::Waited).await?;
    task.status = Status::Downloading;
    Ok(())
}

/// check if torrent lengths are in the limit.
/// Returns true if all lengths are within the limit, false otherwise.
/// # Parameters
/// - `torrents_length_list`: The list of torrent lengths.
/// - `max`: The maximum allowed length.
/// - `selected_file_index`: The indices of the selected files.
fn check(torrent_lengths_list: &[&i64], max: i64, selected_file_index: Option<&[usize]>) -> bool {
    match selected_file_index {
        None => torrent_lengths_list.iter().all(|&&length| length <= max),
        Some(index_list) => index_list
            .iter()
            .all(|&index| *torrent_lengths_list[index] <= max),
    }
}

/// Get the task order, which can customized by selected_file_index
fn get_task_order(
    torrent_lengths_list: &[&i64],
    max: i64,
    selected_file_index: Option<&[usize]>,
) -> Result<Vec<Vec<usize>>, TaskError> {
    if !check(torrent_lengths_list, max, selected_file_index) {
        return Err(TaskError {
            kind: TaskErrorKind::OverSize,
        });
    }

    let mut task_order: Vec<Vec<usize>> = Vec::new();
    let mut current_part: Vec<usize> = Vec::new();
    let mut current_size: i64 = 0;

    match selected_file_index {
        None => {
            for (i, &&length) in torrent_lengths_list.iter().enumerate() {
                if !current_part.is_empty() && current_size + length > max {
                    task_order.push(std::mem::take(&mut current_part));
                    current_size = 0;
                }
                current_part.push(i);
                current_size += length;
            }
        }
        Some(file_index) => {
            for &index in file_index {
                let length = torrent_lengths_list[index];
                if !current_part.is_empty() && current_size + length > max {
                    task_order.push(std::mem::take(&mut current_part));
                    current_size = 0;
                }
                current_part.push(index);
                current_size += length;
            }
        }
    }
    if !current_part.is_empty() {
        task_order.push(current_part);
    }
    Ok(task_order)
}

/// clean waited torrents, always occurs when a task-adding is canceled.
pub async fn clean_waited() -> Result<(), TaskError> {
    let hash_list = qb::get_tag_torrent_list(qb::Tag::Waited).await?;

    if hash_list.is_empty() {
        return Ok(());
    }

    let clean_file_fut = async {
        let clean_task = hash_list.iter().map(|hash| clean(hash)).collect::<Vec<_>>();
        join_all(clean_task).await;
    };

    let clean_qb_fut = async {
        let hash = hash_list.join("|");
        qb::delete(hash.as_str(), true).await?;
        Ok::<(), TaskError>(())
    };

    let (_, qb_clean_result) = join(clean_file_fut, clean_qb_fut).await;
    if let Err(e) = qb_clean_result {
        error!("Failed to clean waited torrents in qBittorrent: {e}");
    }

    Ok(())
}

impl Task {
    pub async fn get_task_map() -> BTreeMap<String, TaskValue> {
        let task_list = TASK_LIST.get().expect("Task list get error").read().await;
        let tasks: Vec<_> = task_list
            .value
            .iter()
            .map(|(hash, value)| async { (hash.clone(), value.0.read().await.clone()) })
            .collect();
        join_all(tasks).await.into_iter().collect()
    }
}
