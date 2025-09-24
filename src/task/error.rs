use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bencode::BencodeError,
    errors::{IntoContextError, CommonError, QbError},
    request::RequestError,
};

/// General task error enum
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TaskError {
    #[error("{0}")]
    Common(
        #[from]
        #[source]
        CommonError,
    ),

    #[error("bencode parse error")]
    Bencode(
        #[from]
        #[source]
        BencodeError,
    ),

    #[error("{}", .0.as_deref().unwrap_or("Unknown upload error".into()))]
    Upload(Option<Cow<'static, str>>),

    // #[error("job_id:{0} missing, upload may finished")]
    // RcloneJobIdMissing(i32),
    #[error("{msg}")]
    Qb {
        msg: Cow<'static, str>,
        #[source]
        source: QbError,
    },

    #[error("File over size limit")]
    OverSize,

    #[error("Request error")]
    Request(
        #[from]
        #[source]
        RequestError,
    ),

    #[error("Task aborted")]
    Abort,
}

impl IntoContextError for QbError {
    type TargetError = TaskError;
    fn into_error(self, msg: impl Into<Cow<'static, str>>) -> Self::TargetError {
        TaskError::Qb {
            msg: msg.into(),
            source: self,
        }
    }
}

/// Error that may occur when a task is added, which is always bind to one single task
/// Record anything that can help user to handle the error (mostly happened because of external reasons)
#[derive(Debug, Serialize, Deserialize, Error)]
#[error("{}\n{}", self.timestamp, self.kind)]
pub struct RuntimeTaskError {
    pub timestamp: String,
    pub kind: RuntimeTaskErrorKind,
    #[source]
    #[serde(skip)]
    source: Option<TaskError>,
}

impl RuntimeTaskError {
    pub fn from_kind(kind: RuntimeTaskErrorKind, source: Option<TaskError>) -> Self {
        let now = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();
        Self {
            timestamp: now,
            kind,
            source,
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy)]
pub enum RuntimeTaskErrorKind {
    #[error("Error during upload task")]
    RuntimeUpload,

    #[error("Failed to launch upload task")]
    LaunchUpload,

    #[error("Torrent not found, probably the torrent having been deleted")]
    TorrentNotFound,

    #[error("Download torrent error")]
    Download,

    #[error("Failed to add next part")]
    AddNextPart,
}

impl RuntimeTaskErrorKind {
    /// whether this error is skipable, if true, the task can be skip to continue
    pub fn skipable(&self) -> bool {
        use RuntimeTaskErrorKind::*;
        match self {
            RuntimeUpload | TorrentNotFound => true,
            _ => false,
        }
    }
}
