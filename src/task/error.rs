use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bencode::BencodeError,
    error::{CommonError, QbError, ResultExt},
    request::RequestError,
};

// TODO: more specific error types
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

    #[error("job_id:{0} missing, upload may finished")]
    RcloneJobIdMissing(i32),
    
    #[error("{msg}")]
    Qb {
        msg: Cow<'static, str>,
        #[source]
        source: QbError,
    },

    #[error("Runtime error")]
    Runtime(#[from] #[source] RuntimeTaskError),

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

impl<V> ResultExt<V> for Result<V, QbError> {
    type TargetError = TaskError;
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, Self::TargetError> {
        self.map_err(|e| TaskError::Qb {
            msg: msg.into(),
            source: e,
        })
    }
}

/// Error that may occur when a task is added, which is always bind to one single task
// TODO: should be more specific
#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy)]
pub enum RuntimeTaskError {
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
