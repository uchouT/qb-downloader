use thiserror::Error;

use crate::{
    bencode::BencodeError,
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

    #[error("bencode parse error")]
    Bencode(
        #[from]
        #[source]
        BencodeError,
    ),

    #[error("Qb error")]
    Qb(
        #[from]
        #[source]
        QbError,
    ),

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
