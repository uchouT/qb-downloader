use std::{error::Error as StdError, fmt::Display};

use crate::error::{CommonError, format_error_cause_chain};

#[derive(Debug)]
pub struct BencodeError {
    pub kind: BencodeErrorKind,
}
#[derive(Debug)]
pub enum BencodeErrorKind {
    Decode,
    Common(CommonError),
    SingleFile,
}

impl Display for BencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BencodeError: {}", format_error_cause_chain(self))
    }
}

impl StdError for BencodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for BencodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Decode => f.write_str("Bencode decode error"),
            Self::Common(ref e) => write!(f, "{e}"),
            Self::SingleFile => f.write_str("Not a multi-file torrent"),
        }
    }
}

impl StdError for BencodeErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Decode => None,
            Self::Common(e) => Some(e),
            Self::SingleFile => None,
        }
    }
}

impl From<CommonError> for BencodeError {
    fn from(value: CommonError) -> Self {
        BencodeError {
            kind: BencodeErrorKind::Common(value),
        }
    }
}
