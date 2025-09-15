use thiserror::Error;

use crate::{
    bencode::error::BencodeError,
    error::{CommonError, QbError},
    request::RequestError,
};

// TODO: more specific error types

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TaskError {
    #[error("{0}")]
    Upload(&'static str),

    #[error("Task not found for hash: {0}")]
    NotFound(String),

    #[error("Download error")]
    Download,

    #[error("{0}")]
    Common(
        #[from]
        #[source]
        CommonError,
    ),

    #[error("bencode parse error: {0}")]
    Bencode(
        #[from]
        #[source]
        BencodeError,
    ),

    #[error("Qb error\ncaused by {0}")]
    Qb(
        #[from]
        #[source]
        QbError,
    ),

    #[error("exceed")]
    OverSize,

    #[error("Request error: {0}")]
    Request(
        #[from]
        #[source]
        RequestError,
    ),
    
    #[error("Task aborted")]
    Abort,
}
