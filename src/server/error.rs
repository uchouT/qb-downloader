use super::{BoxBody, ResultResponse, ServerResult};
use anyhow::Error as AnyError;
use hyper::{Response, StatusCode};
use log::error;
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

    #[error("Bad request multipart")]
    BadRequestMultipart,

    #[error("Unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Any(
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
        ServerError::MissingParams(msg) => Ok(ResultResponse::bad_request(Some(msg.into()))),
        ServerError::BadRequest(_) => Ok(ResultResponse::bad_request(None)),
        ServerError::Unauthorized => Ok(ResultResponse::unauthorized()),
        ServerError::MethodNotAllowed => Ok(ResultResponse::error_with_code(
            StatusCode::METHOD_NOT_ALLOWED,
        )),
        ServerError::BadRequestMultipart => Ok(ResultResponse::bad_request(Some(
            "failed to parse multipart".into(),
        ))),
        ServerError::Any(e) => {
            error!("{:?}", &e);
            Ok(ResultResponse::error_msg(e.to_string()))
        }
    }
}
