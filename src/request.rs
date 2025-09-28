//! http request wrapper
//! # Usage
//! ```
//! request::post("http://example.com")
//!     .basic_auth("username", "password") // to set basic auth
//!     .query([("key", "value")]) // to set query parameters
//!     .form([("key", "value")])  // to set form body, will overwrite any existing body
//!     .json(json!({"key": "value"})) // to set json body, will overwrite the previously set body
//!     .send().await;
//! // or use send_and_then to handle response
//! request::post("http://example.com")
//!     .send_and_then(async |res| {
//!         // handle response
//!     }).await;
//! ```
//! use multipart form
//! ```
//! // create a file part from file path
//! let file_part = FilePart::path("path/to/file.txt").await?; // async read file
//! // or from bytes
//! let file_part = FilePart::bytes(b"file content", "file.txt");
//!
//! let mutipart = request::Multipart::new()
//!     .text("key", "value") // add a text field
//!     .file("file", file_part); // add a file field
//!
//! request::post("http://example.com")
//!     .multipart(multipart)
//!     .send().await;
//! ```
pub mod multipart;

use crate::request::multipart::MultipartBuilder;

use base64::{Engine, engine::general_purpose};
use nyquest_preset::nyquest::{
    AsyncClient, Body, ClientBuilder,
    r#async::{Request, Response},
    header,
};
use serde::Serialize;
use std::{borrow::Cow, collections::HashMap, future::Future, sync::OnceLock, time::Duration};
use thiserror::Error;

static HTTP_CLIENT: OnceLock<AsyncClient> = OnceLock::new();
pub type Res = Response;

/// Initialize the HTTP client
pub async fn init() {
    nyquest_preset::register();
    let client = default_client_builder()
        .build_async()
        .await
        .expect("Failed to create HTTP client");

    HTTP_CLIENT
        .set(client)
        .expect("HTTP client already initialized");
}

/// default client without cookies management
fn default_client_builder() -> ClientBuilder {
    ClientBuilder::default()
        .user_agent("qb-downloader/1.0")
        .request_timeout(Duration::from_secs(30))
        .no_cookies()
}

fn client() -> &'static AsyncClient {
    HTTP_CLIENT.get().expect("HTTP client is not initialized")
}

pub trait MyRequest {
    type RequestBuilder: MyRequestBuilder;
    fn post(url: impl Into<Cow<'static, str>>) -> Self::RequestBuilder;

    fn get(url: impl Into<Cow<'static, str>>) -> Self::RequestBuilder;
}

/// Default implementation of MyRequest
pub struct MyRequestImpl;

impl MyRequest for MyRequestImpl {
    type RequestBuilder = MyRequestBuilderImpl;
    fn post(url: impl Into<Cow<'static, str>>) -> Self::RequestBuilder {
        Self::RequestBuilder {
            url: url.into(),
            method: Method::Post,
            header: HashMap::new(),
            body: None,
        }
    }

    fn get(url: impl Into<Cow<'static, str>>) -> Self::RequestBuilder {
        Self::RequestBuilder {
            url: url.into(),
            method: Method::Get,
            header: HashMap::new(),
            body: None,
        }
    }
}

pub fn post(url: impl Into<Cow<'static, str>>) -> MyRequestBuilderImpl {
    MyRequestImpl::post(url)
}
pub fn get(url: impl Into<Cow<'static, str>>) -> MyRequestBuilderImpl {
    MyRequestImpl::get(url)
}

#[derive(Clone)]
/// Default implementation of MyRequestBuilder
pub struct MyRequestBuilderImpl {
    url: Cow<'static, str>,
    method: Method,
    header: HashMap<Cow<'static, str>, Cow<'static, str>>,
    body: Option<MyBody>,
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
}

#[derive(Clone)]
pub enum MyBody {
    Json(serde_json::Value),
    Form(Vec<(Cow<'static, str>, Cow<'static, str>)>),
    Multipart(MultipartBuilder),
}

pub trait MyRequestBuilder {
    type Err: From<RequestError>;
    /// Set the basic authentication credentials
    fn basic_auth(self, username: &str, password: &str) -> Self;

    fn query<T: Serialize>(self, input: T) -> Self;

    fn header(
        self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self;

    /// set the request body as JSON, will overwrite any existing body,
    /// same as [`MyRequestBuilder::multipart`] and [`MyRequestBuilder::form`]
    fn json<T: Serialize>(self, value: T) -> Self;

    fn multipart(self, parts: MultipartBuilder) -> Self;

    /// set the request body as url-encoded from
    fn form<F, K, V>(self, fields: F) -> Self
    where
        F: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>;

    async fn send_and_then<V, E, Fn: FnOnce(Res) -> Fut, Fut>(self, res: Fn) -> Result<V, E>
    where
        E: From<Self::Err>,
        Fut: Future<Output = Result<V, E>>,
        Self: Sized,
    {
        let response = self.send().await.map_err(E::from)?;
        res(response).await
    }

    async fn send(self) -> Result<Res, Self::Err>;
}

impl MyRequestBuilder for MyRequestBuilderImpl {
    type Err = RequestError;

    fn basic_auth(mut self, username: &str, password: &str) -> Self {
        let value = format!("{username}:{password}");
        let encoded = general_purpose::STANDARD.encode(value);
        let header_value = format!("Basic {encoded}");
        self = self.header(header::AUTHORIZATION, header_value);
        self
    }

    fn query<T: Serialize>(mut self, input: T) -> Self {
        let value = serde_urlencoded::to_string(input).expect("Error query");
        self.url.to_mut().push_str(&format!("?{value}"));
        self
    }

    fn header(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.header.insert(name.into(), value.into());
        self
    }

    /// set the request body as JSON, will overwrite any existing body,
    /// same as [`MyRequestBuilder::multipart`] and [`MyRequestBuilder::form`]
    fn json<T: Serialize>(mut self, value: T) -> Self {
        let json_value = serde_json::to_value(value).expect("Failed to serialize to JSON");
        self.body = Some(MyBody::Json(json_value));
        self
    }

    fn multipart(mut self, parts: MultipartBuilder) -> Self {
        self.body = Some(MyBody::Multipart(parts));
        self
    }

    /// set the request body as url-encoded from
    fn form<F, K, V>(mut self, fields: F) -> Self
    where
        F: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let form_data: Vec<_> = fields
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self.body = Some(MyBody::Form(form_data));
        self
    }

    async fn send(self) -> Result<Res, Self::Err> {
        let url = self.url;
        let mut req = match self.method {
            Method::Get => Request::get(url),
            Method::Post => Request::post(url),
        };
        for (name, value) in self.header {
            req = req.with_header(name, value);
        }

        match self.body {
            None => {}
            Some(mybody) => match mybody {
                MyBody::Json(value) => {
                    req = req.with_body(Body::json(&value).expect("Error serialize json"));
                }
                MyBody::Form(form) => {
                    req = req.with_body(Body::form(form));
                }
                MyBody::Multipart(multipart_builder) => {
                    let multipart = multipart_builder.into_multipart().await?;
                    req = req.with_body(Body::multipart(multipart));
                }
            },
        }

        let res = client().request(req).await.map_err(RequestError::from)?;
        if res.status().is_successful() {
            Ok(res)
        } else {
            Err(RequestError::Response(res.status().code()))
        }
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Network error")]
    Network(
        #[from]
        #[source]
        nyquest::Error,
    ),

    #[error("HTTP error with status code {0}")]
    Response(u16),

    #[error("IO error")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
}
