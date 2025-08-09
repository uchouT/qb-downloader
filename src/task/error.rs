use std::error::Error as StdError;
use std::fmt::Display;

use crate::error::{CommonError, QbError};
#[derive(Debug)]
pub struct TaskError {
    pub kind: TaskErrorKind,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TaskErrorKind {
    Upload(String),
    Download,
    Common(CommonError),
    Qb(QbError),
    Other(String),
    OverSize,
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task error occurred")
    }
}

impl StdError for TaskError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for TaskErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Upload(ref msg) => f.write_str(msg),
            Self::Download => f.write_str("Download error"),
            Self::Common(ref e) => write!(f, "{e}"),
            Self::Qb(ref e) => write!(f, "{e}"),
            Self::Other(ref msg) => f.write_str(msg),
            Self::OverSize => f.write_str("Selected files exceed maximum length"),
        }
    }
}

impl StdError for TaskErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Upload(_) => None,
            Self::Download => None,
            Self::Common(e) => Some(e),
            Self::Qb(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CommonError> for TaskError {
    fn from(value: CommonError) -> Self {
        TaskError {
            kind: TaskErrorKind::Common(value),
        }
    }
}

impl From<reqwest::Error> for TaskError {
    fn from(value: reqwest::Error) -> Self {
        TaskError {
            kind: TaskErrorKind::Common(value.into()),
        }
    }
}

impl From<QbError> for TaskError {
    fn from(value: QbError) -> Self {
        TaskError {
            kind: TaskErrorKind::Qb(value),
        }
    }
}
