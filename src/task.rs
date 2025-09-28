//! qb-downloader task manager
pub mod error;
pub mod handle;
mod resume;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::{Context, Error};
use arc_swap::ArcSwap;
use directories_next::BaseDirs;
use futures_util::future::{join, join_all};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};

use crate::{
    bencode,
    errors::{
        CommonError, ContextedResult, IntoContextedError, QbError, TargetContextedResult, TaskError,
    },
    format_error_chain, qb,
    task::{
        self,
        error::{RuntimeTaskError, RuntimeTaskErrorKind},
        resume::{resume_from_error, skip_task},
    },
    upload::Uploader,
};

const TASK_FILE_NAME: &str = "tasks.json";
const TORRENT_DIR_NAME: &str = "torrents";
static ADD_TORRENT_LOCK: Mutex<()> = Mutex::const_new(());
pub static TASK_LIST: OnceLock<Task> = OnceLock::new();
static TORRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub type TaskMap = BTreeMap<String, Arc<TaskValue>>;
#[derive(Debug)]
pub struct Task {
    pub filepath: PathBuf,
    pub value: RwLock<TaskMap>,
}

/// task value
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskValue {
    pub hash: String,
    pub name: String,
    pub save_path: String,
    pub root_dir: String,
    pub upload_path: String,
    pub total_part_num: usize,
    pub task_order: Vec<Vec<usize>>,
    /// total file count, which is used to set not download.
    pub file_num: usize,
    pub torrent_path: PathBuf,
    pub max_size: i64,
    pub seeding_time_limit: i32,
    pub ratio_limit: f64,
    pub error_info: ArcSwap<Option<RuntimeTaskError>>,
    pub uploader: Uploader,
    /// current task part state
    pub state: RwLock<State>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub current_part_num: usize,
    pub status: Status,
    pub is_seeding: bool,
    pub progress: f64,
}

/// task status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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

    /// More details in [`ErrorInfo`]
    Error,

    Paused,
}

impl TaskValue {
    pub fn state(&self) -> RwLockReadGuard<'_, State> {
        self.state
            .read()
            .expect("Failed to acquire read lock on task status")
    }

    pub fn state_mut(&self) -> RwLockWriteGuard<'_, State> {
        self.state
            .write()
            .expect("Failed to acquire write lock on task status")
    }

    pub fn error_info(&self) -> Arc<Option<RuntimeTaskError>> {
        self.error_info.load().clone()
    }
    pub fn clean_error_info(&self) {
        self.error_info.store(Arc::from(None));
    }
    pub fn set_error_info(&self, error: RuntimeTaskError) {
        self.error_info.store(Arc::from(Some(error)));
    }

    /// Launch the interval task
    /// # Error
    /// may return [`RuntimeTaskError::LaunchUpload`]
    pub async fn run_interval(self: Arc<Self>) -> Result<(), TaskError> {
        info!("Running interval task for: {}", &self.name);
        self.uploader.upload(self.clone()).await?;
        self.state_mut().status = Status::OnTask;
        Ok(())
    }

    /// Check if the upload is complete
    /// # Error
    /// may return [`RuntimeTaskError::RuntimeUpload`]
    pub async fn run_check(self: Arc<Self>) -> Result<(), TaskError> {
        if self.uploader.check(self.clone()).await? {
            self.state_mut().status = Status::Finished;
            info!("Upload completed for task: {}", &self.name);
        }
        Ok(())
    }
}
impl Task {
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
            value: RwLock::new(BTreeMap::new()),
        }
    }

    fn load(task_list: &mut Task) -> Result<(), CommonError> {
        let path = &task_list.filepath;
        if !path.exists() {
            return Ok(());
        }
        let task_file = std::fs::File::open(path)
            .convert_then_add_context(format!("Failed to open task file: {}", path.display()))?;
        let reader = std::io::BufReader::new(task_file);
        let task_map: TaskMap = serde_json::from_reader(reader)
            .convert_then_add_context("Failed to parse task file")?;
        task_list.value = RwLock::new(task_map);
        Ok(())
    }
}

pub fn init(path: Option<PathBuf>) -> Result<(), Error> {
    let mut task_list = Task::new(path);
    Task::load(&mut task_list).with_context(|| {
        format!(
            "Failed to load task list from: {}",
            task_list.filepath.display()
        )
    })?;
    debug!("Task list loaded from: {}", &task_list.filepath.display());
    debug!("Task list content: {:?}", &task_list.value);
    TASK_LIST.set(task_list).expect("failed to set task list");
    TORRENT_DIR
        .set(
            BaseDirs::new()
                .expect("Failed to get data dir")
                .data_dir()
                .join("qb-downloader")
                .join(TORRENT_DIR_NAME),
        )
        .expect("Failed to set torrent directory");
    std::fs::create_dir_all(TORRENT_DIR.get().unwrap())
        .context("Failed to create torrent cache directory")?;
    info!("Task list initialized.");
    Ok(())
}

pub async fn save() -> Result<(), CommonError> {
    let path = filepath();

    let contents = {
        let task_map = task_map().clone();
        serde_json::to_vec(&task_map).convert_then_add_context("Failed to serialize task list")?
    };
    fs::write(path, contents)
        .await
        .convert_then_add_context("Failed to write task list to file")?;
    Ok(())
}

fn filepath() -> &'static Path {
    &TASK_LIST.get().expect("task list not initialized").filepath
}

pub fn task_map() -> RwLockReadGuard<'static, TaskMap> {
    TASK_LIST
        .get()
        .expect("task list not initialized")
        .value
        .read()
        .expect("Failed to acquire read lock on task list")
}

pub fn task_map_mut() -> RwLockWriteGuard<'static, TaskMap> {
    TASK_LIST
        .get()
        .expect("task list not initialized")
        .value
        .write()
        .expect("Failed to acquire read lock on task list")
}

/// Start a task from paused state
pub async fn start(task: Arc<TaskValue>) -> Result<(), TaskError> {
    qb::start(&task.clone().hash)
        .await
        .add_context("Failed to start torrent in qb")?;

    task.state_mut().status = Status::Downloading;
    save().await?;

    info!("Task started for hash: {}", task.hash);
    Ok(())
}

/// Resume a task from Error state
/// # Preconditions
/// - skip is compatible with kind
/// - torrent cache must exist TODO: perhaps in the future add method to recover
pub async fn resume(
    task: Arc<TaskValue>,
    kind: RuntimeTaskErrorKind,
    skip: bool,
) -> Result<(), TaskError> {
    if skip {
        skip_task(task, kind).await
    } else {
        resume_from_error(task, kind).await
    }
}

pub async fn stop(hash: &str) -> Result<(), TaskError> {
    qb::stop(hash)
        .await
        .add_context("Failed to stop torrent in qb")?;

    {
        task_map().get(hash).unwrap().state_mut().status = Status::Paused;
        save().await?;
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
    fs::remove_file(path)
        .await
        .convert_then_add_context(format!("Failed to clean waited torrent file: {hash}"))?;
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
        task_map_mut().remove(hash);
        if let Err(e) = save().await {
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
pub async fn add_torrent<B: Into<Cow<'static, [u8]>>>(
    file: Option<B>,
    url: &str,
    save_path: &str,
) -> Result<String, TaskError> {
    let hash = {
        if let Some(file) = file {
            let file = file.into();
            let hash = bencode::get_hash(&file)?;
            let path = get_torrent_path(&hash);
            fs::write(path, &file)
                .await
                .convert_then_add_context("Failed to write torrent file")?;
            qb::add_by_bytes(url, save_path, file)
                .await
                .add_context("Failed to add torrent by bytes in qb")?;
            hash
        } else {
            let _lock = ADD_TORRENT_LOCK.lock().await;
            let hash = {
                if let Some(hash) = qb::try_parse_hash(url) {
                    qb::add_by_url(url, save_path)
                        .await
                        .add_context("Failed to add torrent by url in qb")?;
                    hash
                } else {
                    qb::add_by_url(url, save_path)
                        .await
                        .add_context("Failed to add torrent by url in qb")?;
                    qb::get_hash()
                        .await
                        .add_context("Failed to get torrent hash in qb")?
                }
            };
            let path = get_torrent_path(&hash);
            qb::export(&hash, &path).await.map_err(|e| {
                if let QbError::Cancelled = e {
                    TaskError::Abort
                } else {
                    TaskError::Qb(
                        e.into_contexted_error("Failed to export torrent file in qb"),
                    )
                }
            })?;
            hash
        }
    };

    qb::add_tag(hash.as_str(), qb::Tag::Waited)
        .await
        .add_context("Failed to add Waited tag in qb")?;
    Ok(hash)
}

pub fn get_torrent_path(hash: &str) -> PathBuf {
    TORRENT_DIR.get().unwrap().join(format!("{hash}.torrent"))
}

/// add task from [`TaskReq`]
#[allow(clippy::too_many_arguments)]
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
    let task_value = TaskValue {
        hash: hash.clone(),
        name,
        root_dir,
        save_path,
        upload_path,
        uploader,
        total_part_num: task_order.len(),
        state: RwLock::new(State {
            current_part_num: 0,
            status: Status::Paused,
            is_seeding: false,
            progress: 0.0,
        }),
        task_order,
        file_num,
        torrent_path,
        max_size,
        seeding_time_limit,
        ratio_limit,
        error_info: ArcSwap::from_pointee(None),
    };
    let task_value = Arc::from(task_value);
    let (set_share_limit_res, launch_res) = join(
        qb::set_share_limit(&hash, ratio_limit, seeding_time_limit),
        launch(0, &hash, task_value.clone()),
    )
    .await;

    set_share_limit_res.add_context("Failed to set share limit")?;
    launch_res?;

    qb::remove_tag(&hash, qb::Tag::Waited)
        .await
        .add_context("Failed to remove Waited tag in qb")?;
    info!("Task added: {hash}");
    task_map_mut().insert(hash, task_value);
    save().await?;
    Ok(())
}

/// launch a task part by index, update current_part_num to the passed index,
/// and set the task status to [`Status::Downloading`] if success.
/// # Preconditions
/// - the task must have been added, with [`Status::Paused`] or [`Status::Error`]
pub async fn launch(index: usize, hash: &str, task: Arc<TaskValue>) -> Result<(), TaskError> {
    qb::set_not_download(hash, task.file_num)
        .await
        .add_context("Failed to set not download in qb")?;
    qb::set_prio(hash, 1, task.task_order.get(index).unwrap())
        .await
        .add_context("Failed to select target file in qb")?;
    qb::start(hash)
        .await
        .add_context("Failed to start torrent in qb")?;
    let mut state = task.state_mut();
    state.current_part_num = index;
    state.status = Status::Downloading;
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
        return Err(TaskError::OverSize);
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
    if task::task_map().is_empty() {
        return Ok(());
    }

    let hash_list = qb::get_tag_torrent_list(qb::Tag::Waited)
        .await
        .add_context("Failed to get waited torrents in qb")?;

    if hash_list.is_empty() {
        return Ok(());
    }

    let clean_file_fut = async {
        let clean_task = hash_list.iter().map(|hash| clean(hash)).collect::<Vec<_>>();
        join_all(clean_task).await;
    };

    let clean_qb_fut = async {
        let hash = hash_list.join("|");
        qb::delete(hash.as_str(), true)
            .await
            .add_context("Failed to delete waited torrents in qb")?;
        Ok::<(), TaskError>(())
    };

    let (_, qb_clean_result) = join(clean_file_fut, clean_qb_fut).await;
    if let Err(e) = qb_clean_result {
        error!("Failed to clean waited torrents in qBittorrent: {e}");
    }

    Ok(())
}
