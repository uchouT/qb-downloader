use thiserror::Error;

pub use crate::qb::error::QbError;
use crate::server::error::ServerError;
pub use crate::task::error::TaskError;
use std::error::Error as StdError;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
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

    #[error("{0}")]
    Common(
        #[from]
        #[source]
        CommonError,
    ),

    #[error("Server error")]
    Server(
        #[from]
        #[source]
        ServerError,
    ),
}

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

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("Io error")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Toml der error")]
    Deserialize(
        #[source]
        #[from]
        toml::de::Error,
    ),

    #[error("Toml ser error")]
    Serialize(
        #[source]
        #[from]
        toml::ser::Error,
    ),

    #[error("Json ser/de error")]
    Json(
        #[source]
        #[from]
        serde_json::Error,
    ),
}
