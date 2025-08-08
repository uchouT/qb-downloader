use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[derive(Debug)]
pub enum Error {
    Qb(String),
    Upload(String),
    Network(String),
    Io(String),
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Qb(msg) => write!(f, "qBittorrent error: {}", msg),
            Error::Upload(msg) => write!(f, "Upload error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl StdError for Error {}

pub fn format_error_chain(err: &dyn StdError) -> String {
    let mut result = format!("Error: {}", err);
    let mut source = err.source();
    let mut level = 1;

    while let Some(err) = source {
        result.push_str(&format!("\n  Caused by ({}): {}", level, err));
        source = err.source();
        level += 1;
    }

    result
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::Other(value.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::Other(value.to_string())
    }
}
