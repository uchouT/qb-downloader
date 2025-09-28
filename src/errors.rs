use thiserror::Error;

use crate::bencode::BencodeError;
pub use crate::qb::QbError;
use crate::request;
pub use crate::task::error::TaskError;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::Debug;

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
pub type CommonError = ContextedError<CommonErrorKind>;

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

/// error wrapper with context message
#[derive(Error)]
#[error("{msg}")]
pub struct ContextedError<E: StdError> {
    msg: Cow<'static, str>,
    #[source]
    source: E,
}

impl<E: StdError + 'static> Debug for ContextedError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_error_chain(self))
    }
}

/// Create a [`ContextedError`] with message and error
pub fn create_contexted_error<E: StdError>(
    msg: impl Into<Cow<'static, str>>,
    e: E,
) -> ContextedError<E> {
    ContextedError {
        msg: msg.into(),
        source: e,
    }
}

/// Wrap Self as [`ContextedError`]
pub trait IntoContextedError<T = Self>: StdError + Into<T>
where
    T: StdError,
{
    fn into_contexted_error(self, msg: impl Into<Cow<'static, str>>) -> ContextedError<T> {
        create_contexted_error(msg, self.into())
    }
}

impl<E: Into<CommonErrorKind> + StdError> IntoContextedError<CommonErrorKind> for E {}

/// Map Result error to [`ContextedError`]
pub trait ContextedResult<V, S: IntoContextedError> {
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, ContextedError<S>>;
    // fn with_context<F: Fn() -> Cow<'static, str>>(self, f: F) -> Result<V, ContextedError<S>>;
}
impl<V, S: IntoContextedError> ContextedResult<V, S> for Result<V, S> {
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, ContextedError<S>> {
        self.convert_then_add_context(msg)
    }
    // fn with_context<F: Fn() -> Cow<'static, str>>(self, f: F) -> Result<V, ContextedError<S>> {
    //     self.convert_then_with_context(f)
    // }
}

/// Map result error to target error, then wrap target error as [`ContextedError`]
pub trait TargetContextedResult<V, E: IntoContextedError<T>, T: StdError> {
    fn convert_then_add_context(
        self,
        msg: impl Into<Cow<'static, str>>,
    ) -> Result<V, ContextedError<T>>;

    fn convert_then_with_context<F: Fn() -> Cow<'static, str>>(
        self,
        f: F,
    ) -> Result<V, ContextedError<T>>;
}
impl<V, E: IntoContextedError<T>, T: StdError> TargetContextedResult<V, E, T> for Result<V, E> {
    fn convert_then_add_context(
        self,
        msg: impl Into<Cow<'static, str>>,
    ) -> Result<V, ContextedError<T>> {
        self.map_err(|e| e.into_contexted_error(msg))
    }
    fn convert_then_with_context<F: Fn() -> Cow<'static, str>>(
        self,
        f: F,
    ) -> Result<V, ContextedError<T>> {
        let msg = f();
        self.convert_then_add_context(msg)
    }
}

#[derive(Debug, Error)]
pub enum AppErrorKind {
    #[error("qb error")]
    Qb(
        #[source]
        #[from]
        QbError,
    ),

    #[error("task error")]
    Task(
        #[from]
        #[source]
        TaskError,
    ),
    #[error(transparent)]
    Common(#[from] CommonError),

    #[error("bencode error")]
    Bencode(
        #[source]
        #[from]
        BencodeError,
    ),

    #[error("multipart error")]
    Mutipart(
        #[source]
        #[from]
        multer::Error,
    ),
}

pub type AppError = ContextedError<AppErrorKind>;
impl<E: Into<AppErrorKind> + StdError> IntoContextedError<AppErrorKind> for E {}
