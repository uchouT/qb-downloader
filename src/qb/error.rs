use thiserror::Error;

use crate::error::CommonError;
use crate::request::RequestError;

#[derive(Debug, Error)]
pub enum QbError {
    #[error("Qb not login")]
    NotLogin,

    #[error("Unsupported Qb version")]
    UnsupportedVersion,

    #[error("torrent exists")]
    NoNewTorrents,

    #[error("torrent cancelled")]
    Cancelled,

    #[error("Request error")]
    Request(
        #[from]
        #[source]
        RequestError,
    ),
    
    #[error("{0}")]
    CommonError(
        #[from]
        #[source]
        CommonError,
    ),
}

impl From<nyquest::Error> for QbError {
    fn from(value: nyquest::Error) -> Self {
        QbError::Request(RequestError::from(value))
    }
}
