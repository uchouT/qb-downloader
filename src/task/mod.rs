//! qb-downloader task manager
pub mod error;
pub mod process;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use directories_next::BaseDirs;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock, time::sleep};

use crate::{
    Entity,
    error::{CommonError, TaskError},
    qb,
    task::error::TaskErrorKind,
    upload::{UploadCheck, Uploader},
};

const TASK_FILE_NAME: &str = "tasks.toml";
const TORRENT_DIR_NAME: &str = "torrents";

pub static TASK_LIST: OnceLock<RwLock<Task>> = OnceLock::new();
static TORRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub type TaskMap = HashMap<String, Arc<TaskItem>>;
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
    pub total_part_num: u32,
    pub current_part_num: u32,
    pub task_order: Vec<Vec<u32>>,

    /// total file count, which is used to set not download.
    pub file_num: u32,
    pub torrent_path: String,
    pub is_seeding: bool,
    pub max_size: u64,
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
        uploader.upload(&mut *guard).await
    }

    /// Check if the upload is complete
    pub async fn run_check(self: Arc<Self>) -> Result<bool, TaskError> {
        let mut guard = self.0.write().await;
        let uploader = guard.uploader;
        uploader.check(&mut *guard).await
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
            value: HashMap::new(),
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
}

pub async fn start(hash: &str) -> Result<(), TaskError> {
    qb::start(hash).await?;
    Task::read(|task_list| {
        if let Some(task) = task_list.get(hash) {
            task.0.blocking_write().status = Status::Downloading;
            Ok(())
        } else {
            let msg = format!("Task not found for hash: {hash}");
            error!("{msg}");
            Err(TaskError {
                kind: error::TaskErrorKind::Other(msg),
            })
        }
    })
    .await?;
    Task::save().await?;
    info!("Task started for hash: {hash}");
    Ok(())
}

pub async fn stop(hash: &str) -> Result<(), TaskError> {
    qb::stop(hash).await?;
    Task::read(|task_list| {
        if let Some(task) = task_list.get(hash) {
            task.0.blocking_write().status = Status::Paused;
            Ok(())
        } else {
            let msg = format!("Task not found for hash: {hash}");
            error!("{msg}");
            Err(TaskError {
                kind: error::TaskErrorKind::Other(msg),
            })
        }
    })
    .await?;
    Task::save().await?;
    info!("Task stopped for hash: {hash}");
    Ok(())
}

/// clean the cached torrent file according to hash
pub async fn clean(hash: &str) -> Result<(), TaskError> {
    fs::remove_file(TORRENT_DIR.get().unwrap().join(format!("{hash}.torrent")))
        .await
        .map_err(CommonError::from)?;
    Ok(())
}

/// Delete task, both qBittorrent task and cached torrent file
pub async fn delete(hash: &str, delete_files: bool) -> Result<(), TaskError> {
    qb::delete(hash, delete_files).await?;
    clean(hash).await?;
    Task::write(|task_list| task_list.remove(hash)).await;
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
    if let Some(file) = file {
        qb::add_by_bytes(url, save_path, file).await?;
    } else {
        qb::add_by_url(url, save_path).await?;
    }
    sleep(std::time::Duration::from_millis(500)).await;
    let hash = qb::get_hash().await?;
    export(hash.as_str(), file).await?;
    qb::add_tag(hash.as_str(), qb::Tag::WAITED).await?;
    Ok(hash)
}

/// Cache torrent file for fastly re-adding
pub async fn export(hash: &str, file_data: Option<&[u8]>) -> Result<(), TaskError> {
    let path = TORRENT_DIR.get().unwrap().join(format!("{hash}.torrent"));
    if let Some(data) = file_data {
        fs::write(path, data).await.map_err(CommonError::from)?;
    } else {
        qb::export(hash, path.to_str().unwrap()).await?;
    }
    Ok(())
}

// TODO: add task

/// launch a task
pub async fn launch(index: u32, hash: &str, task: &mut TaskValue) -> Result<(), TaskError> {
    task.current_part_num = index;
    qb::set_not_download(task).await?;
    qb::set_prio(hash, 1, task.task_order.get(index as usize).unwrap()).await?;
    qb::start(hash).await?;
    qb::remove_tag(hash, qb::Tag::WAITED).await?;
    task.status = Status::Downloading;
    Ok(())
}

/// check if torrent lengths are in the limit.
/// Returns true if all lengths are within the limit, false otherwise.
/// # Parameters
/// - `torrents_length_list`: The list of torrent lengths.
/// - `max`: The maximum allowed length.
/// - `selected_file_index`: The indices of the selected files.
pub fn check(torrent_lengths_list: &[u64], max: u64, selected_file_index: Option<&[u32]>) -> bool {
    match selected_file_index {
        None => torrent_lengths_list.iter().all(|&length| length <= max),
        Some(index_list) => index_list
            .iter()
            .all(|&index| torrent_lengths_list[index as usize] <= max),
    }
}

/// Get the task order, which can customized by selected_file_index
pub fn get_task_order(
    torrent_lengths_list: &[u64],
    max: u64,
    selected_file_index: Option<&[u32]>,
) -> Result<Vec<Vec<u32>>, TaskError> {
    if !check(torrent_lengths_list, max, selected_file_index) {
        return Err(TaskError {
            kind: TaskErrorKind::OverSize,
        });
    }

    let mut task_order: Vec<Vec<u32>> = Vec::new();
    let mut current_part: Vec<u32> = Vec::new();
    let mut current_size: u64 = 0;

    match selected_file_index {
        None => {
            for (i, &length) in torrent_lengths_list.iter().enumerate() {
                if !current_part.is_empty() && current_size + length > max {
                    task_order.push(std::mem::take(&mut current_part));
                    current_size = 0;
                }
                current_part.push(i as u32);
                current_size += length;
            }
        }
        Some(file_index) => {
            for &index in file_index {
                let length = torrent_lengths_list[index as usize];
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
    let hash_list = qb::get_tag_torrent_list(qb::Tag::WAITED).await?;
    for hash in &hash_list {
        clean(hash).await.map_err(|e| {
            let msg = format!("Failed to clean torrent file for hash {hash}: {e}");
            error!("{msg}");
            e
        })?;
    }
    if hash_list.is_empty() {
        return Ok(());
    }
    let hash = hash_list.join("|");
    qb::delete(hash.as_str(), true).await?;
    Ok(())
}

// TODO: return content tree according to hash
