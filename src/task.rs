//! qb-downloader task manager

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::{
    Entity, Error, get_base_dir,
    upload::{UploadCheck, Uploader},
};

pub static TASK_LIST: OnceLock<RwLock<Task>> = OnceLock::new();
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub filepath: PathBuf,
    pub value: HashMap<String, TaskItem>,
}

/// task value
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskItem {
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub async fn run_interval(&mut self) -> Result<(), Error> {
        self.status = Status::OnTask;
        info!("Running interval task for: {}", self.name);

        let uploader = self.uploader.clone();
        uploader.upload(self).await.map_err(|e| {
            let msg = format!("Upload failed: {}", e);
            error!("{}", msg);
            self.status = Status::Error;
            Error::Upload(msg)
        })
    }

    pub async fn run_check(&mut self) -> Result<bool, Error> {
        let uploader = self.uploader.clone();
        uploader.check(self).await.map_err(|e| {
            let msg = format!("Check failed: {}", e);
            error!("{}", msg);
            self.status = Status::Error;
            Error::Upload(msg)
        })
    }
}

impl Entity for Task {
    fn new(path: Option<PathBuf>) -> Self {
        let filepath = if let Some(filepath) = path {
            filepath
        } else {
            get_base_dir().join("tasks.toml")
        };
        Task {
            filepath,
            value: HashMap::new(),
        }
    }

    fn init(path: Option<PathBuf>) -> Result<(), Error> {
        let mut task_list = Self::new(path);
        Self::load(&mut task_list)?;
        info!("Task list loaded from: {}", &task_list.filepath.display());
        debug!("Task list content: {:?}", &task_list.value);
        TASK_LIST
            .set(RwLock::new(task_list))
            .expect("failed to set task list");
        info!("Task list initialized.");
        Ok(())
    }
}
