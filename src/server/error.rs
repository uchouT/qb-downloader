use std::{error::Error as StdError, fmt::Display};

use crate::error::format_error_cause_chain;

#[derive(Debug)]
pub struct ServerError {
    kind: ServerErrorKind,
}

#[derive(Debug)]
pub enum ServerErrorKind {
    MissingParams,
}

impl StdError for ServerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerError: {:?}", format_error_cause_chain(self))
    }
}

impl StdError for ServerErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServerErrorKind::MissingParams => None,
        }
    }
}

impl Display for ServerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerErrorKind::MissingParams => f.write_str("Missing required parameters"),
        }
    }
}

// impl From<hyper::Error> for ServerError {
//     fn from(value: hyper::Error) -> Self {
//         ServerError {
//             kind: ServerErrorKind::Hyper(value),
//         }
//     }
// }

// impl From<std::io::Error> for ServerError {
//     fn from(value: std::io::Error) -> Self {
//         ServerError {
//             kind: ServerErrorKind::Stream(value),
//         }
//     }
// }
