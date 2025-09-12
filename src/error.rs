pub use crate::qb::error::QbError;
use crate::server::error::ServerError;
pub use crate::task::error::TaskError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Qb(QbError),
    Task(TaskError),
    Common(CommonError),
    Other(String),
    Server(ServerError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "App error occurred{}", format_error_cause_chain(self))
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::Qb(ref e) => write!(f, "Qbittorrent error: {e}"),
            Self::Task(ref e) => write!(f, "Task error: {e}"),
            Self::Common(ref e) => write!(f, "{e}"),
            Self::Server(ref e) => write!(f, "Server error: {e}"),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl StdError for ErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Qb(e) => Some(e),
            Self::Task(e) => Some(e),
            Self::Common(e) => Some(e),
            Self::Server(e) => Some(e),
            _ => None,
        }
    }
}

/// print error chain
pub fn format_error_chain(err: impl StdError) -> String {
    let mut result = format!("Error: {err}");
    let mut source = err.source();

    while let Some(err) = source {
        result.push_str(&format!("\n  Caused by: {err}"));
        source = err.source();
    }

    result
}

/// print error souce chain
pub fn format_error_cause_chain(err: impl StdError) -> String {
    let mut result = String::new();
    let mut source = err.source();

    while let Some(err) = source {
        result.push_str(&format!("\n  Caused by: {err}"));
        source = err.source();
    }

    result
}

impl From<CommonError> for Error {
    fn from(value: CommonError) -> Self {
        Error {
            kind: ErrorKind::Common(value),
        }
    }
}

impl From<TaskError> for Error {
    fn from(value: TaskError) -> Self {
        Self {
            kind: ErrorKind::Task(value),
        }
    }
}

impl From<ServerError> for Error {
    fn from(value: ServerError) -> Self {
        Self {
            kind: ErrorKind::Server(value),
        }
    }
}

#[derive(Debug)]
pub enum CommonError {
    Io(std::io::Error),
    /// Connect failed
    Network(nyquest_preset::nyquest::Error),
    /// response is not success
    Response(u16),

    Deserialize(toml::de::Error),
    Serialize(toml::ser::Error),
    Json(serde_json::Error),
}

impl Display for CommonError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Self::Io(ref err) => write!(f, "I/O error: {err}"),
            Self::Network(_) => f.write_str("Request failed due to network"),
            Self::Response(code) => write!(f, "Request failed, response code: {code}"),
            Self::Deserialize(ref e) => write!(f, "TOML deserialization error: {e}"),
            Self::Serialize(ref e) => write!(f, "TOML serialization error: {e}"),
            Self::Json(ref e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl StdError for CommonError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Network(source) => Some(source),
            Self::Deserialize(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<nyquest_preset::nyquest::Error> for CommonError {
    fn from(err: nyquest_preset::nyquest::Error) -> Self {
        CommonError::Network(err)
    }
}

impl From<std::io::Error> for CommonError {
    fn from(value: std::io::Error) -> Self {
        CommonError::Io(value)
    }
}

impl From<toml::de::Error> for CommonError {
    fn from(value: toml::de::Error) -> Self {
        CommonError::Deserialize(value)
    }
}

impl From<toml::ser::Error> for CommonError {
    fn from(value: toml::ser::Error) -> Self {
        CommonError::Serialize(value)
    }
}

impl From<serde_json::Error> for CommonError {
    fn from(value: serde_json::Error) -> Self {
        CommonError::Json(value)
    }
}
