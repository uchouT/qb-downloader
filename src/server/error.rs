use super::{BoxBody, ResultResponse, ServerResult};
use crate::bencode::BencodeError;
use anyhow::Error as AnyError;
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

    #[error("Method not allowed")]
    MethodNotAllowed,

    #[error("Bencode error")]
    Bencode(
        #[from]
        #[source]
        BencodeError,
    ),

    #[error("Bad request multipart")]
    BadRequestMultipart,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("{0}")]
    Unknown(
        #[source]
        #[from]
        AnyError,
    ),
}

impl From<multer::Error> for ServerError {
    fn from(_: multer::Error) -> Self {
        ServerError::BadRequestMultipart
    }
}

pub fn handle(e: ServerError) -> ServerResult<Response<BoxBody>> {
    match e {
        ServerError::MissingParams(msg) => Ok(ResultResponse::bad_request(Some(msg))),
        ServerError::BadRequest(_) => Ok(ResultResponse::bad_request(None)),
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
        ServerError::Unknown(e) => {
            error!("{e:?}");
            Ok(ResultResponse::error())
        }
    }
}
