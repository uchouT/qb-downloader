use super::{BoxBody, ResultResponse, ServerResult};
use crate::{
    bencode::error::{BencodeError, BencodeErrorKind},
    error::{CommonError, QbError, TaskError, format_error_cause_chain},
};
use hyper::{Response, StatusCode};
use log::{error, warn};
use std::{error::Error as StdError, fmt::Display};

#[derive(Debug)]
pub struct ServerError {
    pub kind: ServerErrorKind,
}

#[derive(Debug)]
pub enum ServerErrorKind {
    MissingParams(String),
    BadRequest(Option<hyper::Error>),
    Common(CommonError),
    MethodNotAllowed,
    Qb(QbError),
    Task(TaskError),
    Bencode(BencodeError),
    Multipart(multer::Error),
    Unauthorized,
}

impl StdError for ServerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.kind)
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerError: {}", format_error_cause_chain(self))
    }
}

impl StdError for ServerErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServerErrorKind::BadRequest(err) => {
                if let Some(e) = err {
                    Some(e)
                } else {
                    None
                }
            }
            ServerErrorKind::Common(common_err) => Some(common_err),
            ServerErrorKind::Qb(e) => Some(e),
            ServerErrorKind::Task(e) => Some(e),
            ServerErrorKind::Bencode(e) => Some(e),
            ServerErrorKind::Multipart(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for ServerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerErrorKind::MissingParams(missed_key) => {
                write!(f, "Missing required parameters: {missed_key}")
            }
            ServerErrorKind::BadRequest(err) => {
                if let Some(hyper_err) = err {
                    write!(f, "Bad request {hyper_err}")
                } else {
                    write!(f, "Bad request")
                }
            }
            ServerErrorKind::Common(common_err) => write!(f, "{common_err}"),
            ServerErrorKind::MethodNotAllowed => write!(f, "Method not allowed"),
            ServerErrorKind::Qb(e) => write!(f, "Qb error: {e}"),
            ServerErrorKind::Task(e) => write!(f, "Task error: {e}"),
            ServerErrorKind::Bencode(e) => write!(f, "Bencode error: {e}"),
            ServerErrorKind::Multipart(e) => write!(f, "Multipart error: {e}"),
            ServerErrorKind::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl From<hyper::Error> for ServerError {
    fn from(value: hyper::Error) -> Self {
        ServerError {
            kind: ServerErrorKind::BadRequest(Some(value)),
        }
    }
}

impl From<CommonError> for ServerError {
    fn from(value: CommonError) -> Self {
        ServerError {
            kind: ServerErrorKind::Common(value),
        }
    }
}

impl From<QbError> for ServerError {
    fn from(value: QbError) -> Self {
        Self {
            kind: ServerErrorKind::Qb(value),
        }
    }
}

impl From<TaskError> for ServerError {
    fn from(value: TaskError) -> Self {
        Self {
            kind: ServerErrorKind::Task(value),
        }
    }
}

impl From<BencodeError> for ServerError {
    fn from(value: BencodeError) -> Self {
        Self {
            kind: ServerErrorKind::Bencode(value),
        }
    }
}

impl From<multer::Error> for ServerError {
    fn from(value: multer::Error) -> Self {
        ServerError {
            kind: ServerErrorKind::Multipart(value),
        }
    }
}

pub fn handle(e: ServerError) -> ServerResult<Response<BoxBody>> {
    match e.kind {
        ServerErrorKind::MissingParams(msg) => Ok(ResultResponse::bad_request(Some(msg))),
        ServerErrorKind::BadRequest(_) => Ok(ResultResponse::bad_request(None)),
        ServerErrorKind::Common(_) => Ok(ResultResponse::bad_request(None)),
        ServerErrorKind::Unauthorized => Ok(ResultResponse::unauthorized()),
        ServerErrorKind::MethodNotAllowed => Ok(ResultResponse::error_with_code(
            StatusCode::METHOD_NOT_ALLOWED,
        )),
        ServerErrorKind::Bencode(e) => {
            if let BencodeErrorKind::SingleFile = e.kind {
                warn!("Cannot add single-file torrent");
                Ok(ResultResponse::error_msg("Not a multi-file torrent"))
            } else {
                error!("{e}");
                Ok(ResultResponse::error_msg("torrent parse error"))
            }
        }
        _ => {
            error!("{e}");
            Ok(ResultResponse::error())
        }
    }
}
