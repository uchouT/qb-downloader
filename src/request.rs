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
use base64::{Engine, engine::general_purpose};
use mime_guess::Mime;
use nyquest::{PartBody, r#async::Part};
use nyquest_preset::nyquest::{
    AsyncClient, Body, ClientBuilder,
    r#async::{Request, Response},
    header,
};
use serde::Serialize;
use std::{
    borrow::Cow, collections::HashMap, future::Future, path::Path, sync::OnceLock, time::Duration,
};
use thiserror::Error;

static HTTP_CLIENT: OnceLock<AsyncClient> = OnceLock::new();
type Res = Response;

/// Initialize the HTTP client, which manages cookies automatically
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

fn default_client_builder() -> ClientBuilder {
    ClientBuilder::default()
        .user_agent("qb-downloader/1.0")
        .request_timeout(Duration::from_secs(30))
}

fn client() -> &'static AsyncClient {
    HTTP_CLIENT.get().expect("HTTP client is not initialized")
}

/// Create a client that does not manage cookies, usually for test
async fn cookies_disabled_client() -> AsyncClient {
    default_client_builder()
        .no_cookies()
        .build_async()
        .await
        .unwrap()
}

pub fn post(url: impl Into<Cow<'static, str>>) -> MyRequestBuilder {
    MyRequestBuilder {
        url: url.into(),
        method: Method::Post,
        header: HashMap::new(),
        body: None,
        disable_cookies: false,
    }
}
pub fn get(url: impl Into<Cow<'static, str>>) -> MyRequestBuilder {
    MyRequestBuilder {
        url: url.into(),
        method: Method::Get,
        header: HashMap::new(),
        body: None,
        disable_cookies: false,
    }
}

pub struct MyRequestBuilder {
    url: Cow<'static, str>,
    method: Method,
    header: HashMap<Cow<'static, str>, Cow<'static, str>>,
    body: Option<MyBody>,
    disable_cookies: bool,
}

enum Method {
    Get,
    Post,
}

enum MyBody {
    Json(serde_json::Value),
    Form(Vec<(Cow<'static, str>, Cow<'static, str>)>),
    Multipart(Multipart),
}

pub struct Multipart {
    parts: Vec<Part>,
}

impl IntoIterator for Multipart {
    type Item = Part;
    type IntoIter = std::vec::IntoIter<Part>;

    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

impl Multipart {
    /// create a new empty multipart
    pub fn new() -> Self {
        Self { parts: vec![] }
    }

    /// add a text part
    pub fn text(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        let part = Part::new_with_content_type(name, "text/plain", PartBody::text(value));
        self.parts.push(part);
        self
    }

    /// add a file part
    pub fn file(mut self, name: impl Into<Cow<'static, str>>, file: FilePart) -> Self {
        let content_type = file.mime.to_string();
        let part = Part::new_with_content_type(name, content_type, PartBody::bytes(file.bytes))
            .with_filename(file.filename);
        self.parts.push(part);
        self
    }
}

/// A file part for multipart/form-data
pub struct FilePart {
    bytes: Cow<'static, [u8]>,
    mime: Mime,
    filename: Cow<'static, str>,
}

impl FilePart {
    /// Create a file part from bytes and filename, the mime type is guessed from the filename
    pub fn bytes(
        bytes: impl Into<Cow<'static, [u8]>>,
        filename: impl Into<Cow<'static, str>>,
    ) -> Self {
        let filename = filename.into();
        let mime = mime_guess::from_path(Path::new(filename.as_ref())).first_or_octet_stream();
        Self {
            bytes: bytes.into(),
            mime,
            filename,
        }
    }

    /// Create a file part from a file path, the mime type is guessed from the file extension
    pub async fn path(path: &Path) -> Result<Self, RequestError> {
        let bytes = tokio::fs::read(path).await?;
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Ok(Self {
            bytes: bytes.into(),
            mime,
            filename: filename.into(),
        })
    }
}

impl MyRequestBuilder {
    /// Disable cookie management for this request
    pub fn disable_cookie(mut self) -> Self {
        self.disable_cookies = true;
        self
    }

    /// Set the basic authentication credentials
    pub fn basic_auth(mut self, username: &str, password: &str) -> Self {
        let value = format!("{username}:{password}");
        let encoded = general_purpose::STANDARD.encode(value);
        let header_value = format!("Basic {encoded}");
        self.header
            .insert(header::AUTHORIZATION.into(), header_value.into());
        self
    }

    pub fn query<T: Serialize>(mut self, input: T) -> Self {
        let value = serde_urlencoded::to_string(input).expect("Error query");
        self.url.to_mut().push_str(&format!("?{value}"));
        self
    }

    // pub fn header(
    //     mut self,
    //     name: impl Into<Cow<'static, str>>,
    //     value: impl Into<Cow<'static, str>>,
    // ) -> Self {
    //     self.header.insert(name.into(), value.into());
    //     self
    // }

    /// set the request body as JSON, will overwrite any existing body,
    /// same as [`MyRequestBuilder::multipart`] and [`MyRequestBuilder::form`]
    pub fn json<T: Serialize>(mut self, value: T) -> Self {
        let json_value = serde_json::to_value(value).expect("Failed to serialize to JSON");
        self.body = Some(MyBody::Json(json_value));
        self
    }

    pub fn multipart(mut self, parts: Multipart) -> Self {
        self.body = Some(MyBody::Multipart(parts));
        self
    }

    /// set the request body as url-encoded from
    pub fn form<F, K, V>(mut self, fields: F) -> Self
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

    pub async fn send_and_then<V, E, Fn: FnOnce(Res) -> Fut, Fut>(self, res: Fn) -> Result<V, E>
    where
        Fut: Future<Output = Result<V, E>>,
        E: From<RequestError>,
    {
        let response = self.send().await?;
        res(response).await
    }

    pub async fn send(self) -> Result<Res, RequestError> {
        let mut req = match self.method {
            Method::Get => Request::get(self.url),
            Method::Post => Request::post(self.url),
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
                MyBody::Multipart(multi) => {
                    req = req.with_body(Body::multipart(multi));
                }
            },
        }

        let res = if self.disable_cookies {
            cookies_disabled_client().await.request(req).await?
        } else {
            client().request(req).await?
        };
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
