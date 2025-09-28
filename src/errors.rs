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
#[derive(Debug, Error)]
#[error("{msg}")]
pub struct ContextedError<E: StdError> {
    msg: Cow<'static, str>,
    #[source]
    source: E,
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
impl IntoContextedError for QbError {}

/// Map Result error to [`ContextedError`]
pub trait ContextedResult<V, S: IntoContextedError> {
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, ContextedError<S>>;
}
impl<V, S: IntoContextedError> ContextedResult<V, S> for Result<V, S> {
    fn add_context(self, msg: impl Into<Cow<'static, str>>) -> Result<V, ContextedError<S>> {
        self.map_err(|e| e.into_contexted_error(msg))
    }
}

/// Map result error to target error, then wrap target error as [`ContextedError`]
pub trait TargetContextedResult<V, E: IntoContextedError<T>, T: StdError> {
    fn convert_then_add_context(
        self,
        msg: impl Into<Cow<'static, str>>,
    ) -> Result<V, ContextedError<T>>;
}
impl<V, E: IntoContextedError<T>, T: StdError> TargetContextedResult<V, E, T> for Result<V, E> {
    fn convert_then_add_context(
        self,
        msg: impl Into<Cow<'static, str>>,
    ) -> Result<V, ContextedError<T>> {
        self.map_err(|e| e.into_contexted_error(msg))
    }
}

impl<E: Into<CommonErrorKind> + StdError> IntoContextedError<CommonErrorKind> for E {}
