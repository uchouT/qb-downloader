use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[derive(Debug)]
pub enum Error {
    Qb(Box<dyn StdError + Send + Sync>),
    Network(String),
    Other(Box<dyn StdError + Send + Sync>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Qb(e) => write!(f, "qBittorrent error: {}", e),
            Error::Network(msg) => write!(f, "{}", msg),
            Error::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Qb(e) => Some(e.as_ref()),
            Error::Network(_) => None,
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
