use std::borrow::Cow;

use crate::errors::AppError;

use super::{BoxBody, ResultResponse, ServerResult};
use hyper::{Response, StatusCode};
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Missing params: {0}")]
    MissingParams(&'static str),

    #[error("Bad request")]
    BadRequestHyper(
        #[source]
        #[from]
        hyper::Error,
    ),

    #[error("{}", .0.as_deref().unwrap_or_default())]
    BadRequest(Option<Cow<'static, str>>),

    #[error("Method not allowed")]
    MethodNotAllowed,

    #[error("Bad request multipart")]
    BadRequestMultipart,

    #[error("Unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Any(#[from] AppError),

    #[error("{0}")]
    Internal(Cow<'static, str>),
}

impl From<multer::Error> for ServerError {
    fn from(_: multer::Error) -> Self {
        ServerError::BadRequestMultipart
    }
}

pub fn handle(e: ServerError) -> ServerResult<Response<BoxBody>> {
    match e {
        ServerError::MissingParams(msg) => Ok(ResultResponse::bad_request(Some(msg.into()))),
        ServerError::BadRequestHyper(_) => Ok(ResultResponse::bad_request(None)),
        ServerError::Unauthorized => Ok(ResultResponse::unauthorized()),
        ServerError::MethodNotAllowed => Ok(ResultResponse::error_with_code(
            StatusCode::METHOD_NOT_ALLOWED,
        )),
        ServerError::BadRequest(msg) => Ok(ResultResponse::bad_request(msg)),
        ServerError::BadRequestMultipart => Ok(ResultResponse::bad_request(Some(
            "failed to parse multipart".into(),
        ))),
        ServerError::Any(e) => {
            error!("{:?}", &e);
            Ok(ResultResponse::error_msg(e.to_string()))
        }
        ServerError::Internal(msg) => {
            error!("{msg}");
            Ok(ResultResponse::error_msg(msg))
        }
    }
}

impl ServerError {
    pub fn create_internal(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal(msg.into())
    }
}
