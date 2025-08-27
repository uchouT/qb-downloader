//! This module provides tools when building the server API.
pub mod auth;
pub mod config_action;
pub mod login_action;
pub mod task_action;
pub mod test_action;
pub mod torrent_action;
pub mod asset;
use super::{BoxBody, Req};
use crate::{
    error::CommonError,
    server::error::{ServerError, ServerErrorKind},
};
use http_body_util::BodyExt;
use hyper::{Response, body::Bytes, header};
use multer::Multipart;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::from_slice;
use std::{collections::HashMap, str::FromStr};

pub type ServerResult<T> = std::result::Result<T, ServerError>;

pub trait Action: Send + Sync {
    fn execute(&self, req: Req) -> impl Future<Output = ServerResult<Response<BoxBody>>> + Send;
    fn needs_auth(&self) -> bool {
        true
    }
    /// default needs auth
    fn auth(&self, req: &Req) -> ServerResult<()> {
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION)
            && !auth_header.is_empty()
                && auth::authorize(auth_header.to_str().unwrap_or_default()) {
                    return Ok(());
                }
        Err(ServerError {
            kind: ServerErrorKind::Unauthorized,
        })
    }
}

/// Extracts a parameter from the request URL query
/// # Example
/// ```
/// let params = get_param_map(&req).unwrap();
/// let value: u32 = get_option_param(&params, "key").await;
/// ```
pub fn get_option_param<T: FromStr>(params: &HashMap<String, String>, key: &str) -> Option<T> {
    let value = params.get(key)?;
    value.parse::<T>().ok()
}

/// Extracts a parameter from the request URL query, returning a default value if the parameter is not found.
pub fn get_param_or<T: FromStr>(params: &HashMap<String, String>, key: &str, default: T) -> T {
    get_option_param(params, key).unwrap_or(default)
}

/// Extracts a required parameter
/// # Error
/// Returns a [MissingParams](super::error::ServerErrorKind::MissingParams) if the parameter is not found.
pub fn get_required_param<T: FromStr>(
    params: &HashMap<String, String>,
    key: &'static str,
) -> ServerResult<T> {
    get_option_param(params, key).ok_or(ServerError {
        kind: ServerErrorKind::MissingParams(key),
    })
}

/// Extracts parameters from the request query string into a HashMap.
pub fn get_param_map(req: &Req) -> Option<HashMap<String, String>> {
    let query = req.uri().query()?;
    Some(url_decode(query))
}
/// Accept a query string (e.g., "key1=value1&key2=value2"), return a HashMap
fn url_decode(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next()?.to_string();
            Some((key, value))
        })
        .collect::<HashMap<String, String>>()
}

/// Zero-copy get json body, always cooperate with [from_json]
/// # Example
/// ```
/// let bytes = get_json_body(req).await?;
/// let value: MyStruct = from_json(&bytes)?;
/// ```
pub async fn get_json_body(req: Req) -> ServerResult<Bytes> {
    let data = req.into_body().collect().await?.to_bytes();
    Ok(data)
}

pub fn from_json<'de, T: Deserialize<'de>>(data: &'de Bytes) -> ServerResult<T> {
    let value: T = from_slice(data).map_err(CommonError::from)?;
    Ok(value)
}

pub async fn from_json_owned<T: DeserializeOwned>(req: Req) -> ServerResult<T> {
    let data = get_json_body(req).await?;
    from_json(&data)
}

pub trait ReqExt {
    fn is_multipart(&self) -> bool;
    fn into_multipart<'a>(self) -> ServerResult<Multipart<'a>>;
}
impl ReqExt for Req {
    fn is_multipart(&self) -> bool {
        if let Some(content_type) = self.headers().get(hyper::header::CONTENT_TYPE)
            && let Ok(content_type_str) = content_type.to_str() {
                return content_type_str.starts_with("multipart/");
            }
        false
    }
    fn into_multipart<'a>(self) -> ServerResult<Multipart<'a>> {
        let content_type = self
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .ok_or(ServerError {
                kind: ServerErrorKind::BadRequest(None),
            })?
            .to_str()
            .map_err(|_| ServerError {
                kind: ServerErrorKind::BadRequest(None),
            })?;

        let boundary = multer::parse_boundary(content_type).map_err(|_| ServerError {
            kind: ServerErrorKind::BadRequest(None),
        })?;
        let stream = self.into_body().into_data_stream();
        Ok(Multipart::new(stream, boundary))
    }
}
