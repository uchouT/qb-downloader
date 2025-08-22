use crate::error::{CommonError, format_error_cause_chain};
use std::error::Error as StdError;
use std::fmt::Display;
#[derive(Debug)]
pub struct QbError {
    pub kind: QbErrorKind,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum QbErrorKind {
    NotLogin,

    Other(String),
    NoNewTorrents,

    Common(CommonError),
}

impl Display for QbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Qbittorrent error occurred{}",
            format_error_cause_chain(self)
        )
    }
}

impl StdError for QbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for QbErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NotLogin => f.write_str("qBittorrent haven't logined"),
            Self::Other(ref msg) => f.write_str(msg),
            Self::NoNewTorrents => f.write_str("No new torrents found"),
            Self::Common(ref err) => write!(f, "{err}"),
        }
    }
}

impl StdError for QbErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Common(err) => Some(err),
            _ => None,
        }
    }
}

impl From<CommonError> for QbError {
    fn from(err: CommonError) -> Self {
        QbError {
            kind: QbErrorKind::Common(err),
        }
    }
}

impl From<reqwest::Error> for QbError {
    fn from(err: reqwest::Error) -> Self {
        QbError {
            kind: QbErrorKind::Common(CommonError::from(err)),
        }
    }
}

impl From<std::io::Error> for QbError {
    fn from(err: std::io::Error) -> Self {
        QbError {
            kind: QbErrorKind::Common(CommonError::from(err)),
        }
    }
}
