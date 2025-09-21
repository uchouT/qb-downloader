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

use self::multipart::Multipart;

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

/// Default implementation of MyRequestBuilder
pub struct MyRequestBuilderImpl {
    url: Cow<'static, str>,
    method: Method,
    header: HashMap<Cow<'static, str>, Cow<'static, str>>,
    body: Option<MyBody>,
}

pub trait MyRequestBuilderAccessor {
    /// Get the URL of the request
    fn url_mut(&mut self) -> Cow<'static, str>;

    fn method(&self) -> Method;

    fn headers_mut(&mut self) -> &mut HashMap<Cow<'static, str>, Cow<'static, str>>;

    fn take_body(&mut self) -> Option<MyBody>;

    fn body_mut(&mut self) -> &mut Option<MyBody>;
}

impl MyRequestBuilderAccessor for MyRequestBuilderImpl {
    /// Get the URL of the request
    fn url_mut(&mut self) -> Cow<'static, str> {
        std::mem::take(&mut self.url)
    }

    fn method(&self) -> Method {
        self.method
    }

    fn headers_mut(&mut self) -> &mut HashMap<Cow<'static, str>, Cow<'static, str>> {
        &mut self.header
    }

    fn take_body(&mut self) -> Option<MyBody> {
        self.body.take()
    }

    fn body_mut(&mut self) -> &mut Option<MyBody> {
        &mut self.body
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
}

pub enum MyBody {
    Json(serde_json::Value),
    Form(Vec<(Cow<'static, str>, Cow<'static, str>)>),
    Multipart(Multipart),
}

pub trait MyRequestBuilder: MyRequestBuilderAccessor {
    type Err: From<RequestError>;
    /// Set the basic authentication credentials
    fn basic_auth(&mut self, username: &str, password: &str) -> &mut Self {
        let value = format!("{username}:{password}");
        let encoded = general_purpose::STANDARD.encode(value);
        let header_value = format!("Basic {encoded}");
        self.headers_mut()
            .insert(header::AUTHORIZATION.into(), header_value.into());
        self
    }

    fn query<T: Serialize>(&mut self, input: T) -> &mut Self {
        let value = serde_urlencoded::to_string(input).expect("Error query");
        self.url_mut().to_mut().push_str(&format!("?{value}"));
        self
    }

    fn header(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.headers_mut().insert(name.into(), value.into());
        self
    }

    /// set the request body as JSON, will overwrite any existing body,
    /// same as [`MyRequestBuilder::multipart`] and [`MyRequestBuilder::form`]
    fn json<T: Serialize>(&mut self, value: T) -> &mut Self {
        let json_value = serde_json::to_value(value).expect("Failed to serialize to JSON");
        *self.body_mut() = Some(MyBody::Json(json_value));
        self
    }

    fn multipart(&mut self, parts: Multipart) -> &mut Self {
        *self.body_mut() = Some(MyBody::Multipart(parts));
        self
    }

    /// set the request body as url-encoded from
    fn form<F, K, V>(&mut self, fields: F) -> &mut Self
    where
        F: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let form_data: Vec<_> = fields
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        *self.body_mut() = Some(MyBody::Form(form_data));
        self
    }

    async fn send_and_then<V, E, Fn: FnOnce(Res) -> Fut, Fut>(&mut self, res: Fn) -> Result<V, E>
    where
        E: From<Self::Err>,
        Fut: Future<Output = Result<V, E>>,
    {
        let response = self.send().await.map_err(E::from)?;
        res(response).await
    }

    async fn send(&mut self) -> Result<Res, Self::Err> {
        let mut req = match self.method() {
            Method::Get => Request::get(self.url_mut()),
            Method::Post => Request::post(self.url_mut()),
        };

        for (name, value) in self.headers_mut().drain() {
            req = req.with_header(name, value);
        }

        match self.take_body() {
            None => {}
            Some(mybody) => match mybody {
                MyBody::Json(value) => {
                    req = req.with_body(Body::json(&value).expect("Error serialize json"));
                }
                MyBody::Form(form) => {
                    req = req.with_body(Body::form(form));
                }
                MyBody::Multipart(multi) => {
                    req = req.with_body(Body::multipart(multi));
                }
            },
        }

        let res = client().request(req).await.map_err(RequestError::from)?;
        if res.status().is_successful() {
            Ok(res)
        } else {
            Err(RequestError::Response(res.status().code())).map_err(Self::Err::from)
        }
    }
}

impl MyRequestBuilder for MyRequestBuilderImpl {
    type Err = RequestError;
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
