use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[derive(Debug)]
pub enum Error {
    Qb(Box<dyn StdError + Send + Sync>),
    Network(String),
    Io(String),
    Other(Box<dyn StdError + Send + Sync>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Qb(e) => write!(f, "qBittorrent error: {}", e),
            Error::Network(msg) => write!(f, "{}", msg),
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Qb(e) => Some(e.as_ref()),
            Error::Network(_) => None,
            Error::Io(_) => None,
            Error::Other(e) => Some(e.as_ref()),
        }
    }
}

pub fn format_error_chain(err: &dyn std::error::Error) -> String {
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
        Error::Other(Box::new(value))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::Other(Box::new(value))
    }
}
