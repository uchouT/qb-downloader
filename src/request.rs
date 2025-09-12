//! http request module

use crate::error::CommonError;

use base64::{Engine, engine::general_purpose};
use nyquest_preset::nyquest::{
    AsyncClient, Body, ClientBuilder,
    r#async::{Request, Response},
    header,
};
use serde::Serialize;
use std::{borrow::Cow, future::Future, sync::OnceLock, time::Duration};
static HTTP_CLIENT: OnceLock<AsyncClient> = OnceLock::new();

pub async fn init() {
    nyquest_preset::register();
    let client = ClientBuilder::default()
        .user_agent("qb-downloader/1.0")
        .request_timeout(Duration::from_secs(30))
        .build_async()
        .await
        .expect("Failed to create HTTP client");
    HTTP_CLIENT.set(client);
}
fn get_client() -> &'static AsyncClient {
    HTTP_CLIENT.get().expect("HTTP client is not initialized")
}

pub fn post(url: String) -> Request {
    Request::post(url)
}
pub fn get(url: String) -> Request {
    Request::get(url)
}

// Never used
// pub fn delete<T: AsRef<str>>(url: T) -> RequestBuilder {
//     get_client().delete(url.as_ref())
// }

// pub fn put<T: AsRef<str>>(url: T) -> RequestBuilder {
//     get_client().put(url.as_ref())
// }

pub trait RequestExt {
    type Res;
    fn disable_cookie(self) -> Self;
    fn basic_auth(self, username: &str, password: &str) -> Self;
    fn header(
        self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self;
    fn json<T: Serialize>(self, value: &T) -> Self;
    fn multipart(self, parts: impl IntoIterator<Item = Part<S>>) -> Self;
    fn form(self, fields: impl IntoIterator<Item = (Cow<'static, str>, Cow<'static, str>)>)
    -> Self;
    fn then<V, E, F: FnOnce(Self::Res) -> Fut, Fut: Future<Output = Result<V, E>>>(
        self,
        f: F,
    ) -> impl Future<Output = Result<V, E>>
    where
        CommonError: Into<E>;
}

impl RequestExt for Request {
    type Res = Response;
    fn disable_cookie(self) -> Self {
        self.with_header(header::COOKIE, "")
    }

    fn basic_auth(self, username: &str, password: &str) -> Self {
        let value = format!("{username}:{password}");
        let value = general_purpose::STANDARD.encode(value);
        let header_value = format!("Basic {value}");
        self.with_header(header::AUTHORIZATION, header_value)
    }

    fn header(
        self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.with_header(name, value)
    }

    fn json<T: Serialize>(self, value: &T) -> Result<Self, CommonError> {
        self.with_body(Body::json(value)?)
    }

    fn multipart(self, parts: impl IntoIterator<Item = Part<S>>) -> Self {
        self.with_body(Body::multipart(parts))
    }

    fn form(
        self,
        fields: impl IntoIterator<Item = (Cow<'static, str>, Cow<'static, str>)>,
    ) -> Self {
        self.with_body(Body::form(fields))
    }

    async fn then<V, E, F: FnOnce(Self::Res) -> Fut, Fut: Future<Output = Result<V, E>>>(
        self,
        res: F,
    ) -> Result<V, E>
    where
        CommonError: Into<E>,
    {
        let result = get_client().request(self).await;
        match result {
            Ok(response) => {
                if !response.status().is_successful() {
                    return Err(CommonError::Response(response.status().code()).into());
                }
                res(response).await
            }
            Err(e) => Err(CommonError::Network(e).into()),
        }
    }
}
