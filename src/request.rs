//! http request module
use base64::{Engine, engine::general_purpose};
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

pub fn post(url: impl Into<Cow<'static, str>>) -> MyRequestBuilder {
    MyRequestBuilder {
        url: url.into(),
        method: Method::Post,
        header: HashMap::new(),
        body: None,
    }
}
pub fn get(url: impl Into<Cow<'static, str>>) -> MyRequestBuilder {
    MyRequestBuilder {
        url: url.into(),
        method: Method::Get,
        header: HashMap::new(),
        body: None,
    }
}

pub struct MyRequestBuilder {
    url: Cow<'static, str>,
    method: Method,
    header: HashMap<Cow<'static, str>, Cow<'static, str>>,
    body: Option<MyBody>,
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
    pub async fn file(
        mut self,
        name: impl Into<Cow<'static, str>>,
        path: impl AsRef<Path>,
    ) -> Result<Self, RequestError> {
        let path = path.as_ref();

        let file = tokio::fs::read(path).await?;
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();

        let part = Part::new_with_content_type(name, mime.to_string(), PartBody::bytes(file))
            .with_filename(filename);
        self.parts.push(part);
        Ok(self)
    }
}

impl MyRequestBuilder {
    pub fn disable_cookie(mut self) -> Self {
        self.header.remove(header::COOKIE);
        self
    }

    pub fn basic_auth(mut self, username: &str, password: &str) -> Self {
        let value = format!("{username}:{password}");
        let value = general_purpose::STANDARD.encode(value);
        let header_value = format!("Basic {value}");
        self.header
            .insert(header::AUTHORIZATION.into(), header_value.into());
        self
    }

    pub fn header(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.header.insert(name.into(), value.into());
        self
    }

    pub fn json<T: Serialize>(mut self, value: T) -> Self {
        if self.body.is_some() {
            panic!("body already set"); // should never happen after development
        }
        let json_value = serde_json::to_value(value).expect("Failed to serialize to JSON");
        self.body = Some(MyBody::Json(json_value));
        self
    }

    pub fn multipart(mut self, parts: Multipart) -> Self {
        if self.body.is_some() {
            panic!("body already set");
        }
        self.body = Some(MyBody::Multipart(parts));
        self
    }

    pub fn form<F: IntoIterator<Item = (Cow<'static, str>, Cow<'static, str>)>>(
        mut self,
        fields: F,
    ) -> Self {
        if self.body.is_some() {
            panic!("body already set");
        }
        let form_data: Vec<_> = fields.into_iter().collect();
        self.body = Some(MyBody::Form(form_data));
        self
    }

    pub fn query<T: Serialize>(mut self, input: T) -> Self {
        let value = serde_urlencoded::to_string(input).expect("Error query");
        self.url.to_mut().push_str(&format!("?{value}"));
        self
    }

    pub async fn send_and_then<V, E, Fn: FnOnce(Res) -> Fut, Fut: Future<Output = Result<V, E>>>(
        self,
        res: Fn,
    ) -> Result<V, E>
    where
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

        let res = get_client().request(req).await?;
        if res.status().is_successful() {
            Ok(res)
        } else {
            Err(RequestError::Response(res.status().code()))
        }
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Network error: {0}")]
    Network(#[from] nyquest::Error),

    #[error("HTTP error with status code {0}")]
    Response(u16),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
