use super::{BoxBody, ResultResponse, ServerResult};
use crate::{
    bencode::error::BencodeError,
    error::{CommonError, QbError, TaskError},
};
use hyper::{Response, StatusCode};
use log::{error, warn};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Missing params: {0}")]
    MissingParams(&'static str),

    #[error("Bad request")]
    BadRequest(
        #[source]
        #[from]
        hyper::Error,
    ),

    #[error("{0}")]
    Common(
        #[source]
        #[from]
        CommonError,
    ),

    #[error("Method not allowed")]
    MethodNotAllowed,

    #[error("Qb error")]
    Qb(
        #[from]
        #[source]
        QbError,
    ),

    #[error("Task error")]
    Task(
        #[from]
        #[source]
        TaskError,
    ),

    #[error("Bencode error")]
    Bencode(
        #[from]
        #[source]
        BencodeError,
    ),

    #[error("Multipart error")]
    Multipart(
        #[from]
        #[source]
        multer::Error,
    ),

    #[error("Bad request multipart")]
    BadRequestMultipart,

    #[error("Unauthorized")]
    Unauthorized,
}

pub fn handle(e: ServerError) -> ServerResult<Response<BoxBody>> {
    match e {
        ServerError::MissingParams(msg) => Ok(ResultResponse::bad_request(Some(msg))),
        ServerError::BadRequest(_) => Ok(ResultResponse::bad_request(None)),
        ServerError::Common(_) => Ok(ResultResponse::bad_request(None)),
        ServerError::Unauthorized => Ok(ResultResponse::unauthorized()),
        ServerError::MethodNotAllowed => Ok(ResultResponse::error_with_code(
            StatusCode::METHOD_NOT_ALLOWED,
        )),
        ServerError::BadRequestMultipart => Ok(ResultResponse::bad_request(Some(
            "failed to parse multipart",
        ))),
        ServerError::Bencode(e) => {
            if let BencodeError::SingleFile = e {
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
