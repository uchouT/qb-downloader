use thiserror::Error;

pub use crate::qb::QbError;
use crate::request;
pub use crate::task::error::TaskError;
use std::borrow::Cow;
use std::error::Error as StdError;

/// print error chain
pub fn format_error_chain(err: impl StdError) -> String {
    let mut result = format!("{err}");
    let mut source = err.source();

    while let Some(err) = source {
        result.push_str(&format!("\n  Caused by: {err}"));
        source = err.source();
    }

    result
}

type IoError = std::io::Error;
type DeError = toml::de::Error;
type SerError = toml::ser::Error;
type JsonError = serde_json::Error;

/// Unexpected and unhandleable error that may occur in multiple places, with context message.
#[derive(Debug, Error)]
#[error("{msg}")]
pub struct CommonError {
    pub msg: Cow<'static, str>,
    #[source]
    pub kind: CommonErrorKind,
}
#[derive(Debug, Error)]
pub enum CommonErrorKind {
    #[error("Io error")]
    Io(
        #[from]
        #[source]
        IoError,
    ),

    #[error("Toml der error")]
    Deserialize,

    #[error("Toml ser error")]
    Serialize(
        #[source]
        #[from]
        SerError,
    ),

    #[error("Json ser/de error")]
    Json(
        #[source]
        #[from]
        JsonError,
    ),

    #[error("Http request error")]
    Request(
        #[from]
        #[source]
        request::RequestError,
    ),
}

impl From<DeError> for CommonErrorKind {
    fn from(_: DeError) -> Self {
        CommonErrorKind::Deserialize
    }
}

/// Add context to error and turn into TargetError
pub trait IntoContextError: StdError {
    type TargetError;
    fn into_error(self, msg: impl Into<Cow<'static, str>>) -> Self::TargetError;
}

/// Add context to a Result, whose Err implements [`IntoContextError`]
pub trait ResultExt<V, S: IntoContextError> {
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, S::TargetError>;
}

impl<V, S> ResultExt<V, S> for Result<V, S>
where
    S: IntoContextError,
{
    fn add_context(
        self,
        msg: impl Into<Cow<'static, str>>,
    ) -> Result<V, <S as IntoContextError>::TargetError> {
        self.map_err(|e| e.into_error(msg.into()))
    }
}

impl<E> IntoContextError for E
where
    E: Into<CommonErrorKind> + StdError,
{
    type TargetError = CommonError;
    fn into_error(self, msg: impl Into<Cow<'static, str>>) -> Self::TargetError {
        CommonError {
            msg: msg.into(),
            kind: self.into(),
        }
    }
}
// impl<V, E> ResultExt<V> for Result<V, E>
// where
//     E: Into<CommonErrorKind> + StdError,
// {
//     type TargetError = CommonError;
//     fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, CommonError> {
//         self.map_err(|e| CommonError {
//             msg: msg.into(),
//             kind: e.into(),
//         })
//     }
// }
