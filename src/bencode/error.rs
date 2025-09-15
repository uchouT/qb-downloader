use crate::error::CommonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BencodeError {
    #[error("Failed to decode .torrent file")]
    Decode,

    #[error("{0}")]
    Common(
        #[from]
        #[source]
        CommonError,
    ),

    #[error("Single file torrent is not supported")]
    SingleFile,
}

impl From<bendy::decoding::Error> for BencodeError {
    fn from(_: bendy::decoding::Error) -> Self {
        BencodeError::Decode
    }
}