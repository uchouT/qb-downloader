//! http request module

use crate::error::CommonError;

use reqwest::{Client, RequestBuilder, Response, cookie::Jar, header};
use std::{
    future::Future,
    sync::{Arc, OnceLock},
    time::Duration,
};
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        let jar = Arc::new(Jar::default());
        Client::builder()
            .user_agent("qb-downloader-rust/1.0")
            .cookie_provider(jar)
            // .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(0)
            .build()
            .expect("Failed to create HTTP client")
    })
}

pub fn post<T: AsRef<str>>(url: T) -> RequestBuilder {
    get_client().post(url.as_ref())
}

pub fn get<T: AsRef<str>>(url: T) -> RequestBuilder {
    get_client().get(url.as_ref())
}

pub fn delete<T: AsRef<str>>(url: T) -> RequestBuilder {
    get_client().delete(url.as_ref())
}

pub fn put<T: AsRef<str>>(url: T) -> RequestBuilder {
    get_client().put(url.as_ref())
}

pub trait RequestBuilderExt {
    fn disable_cookie(self) -> RequestBuilder;
    fn then<V, E, F: FnOnce(Response) -> Fut, Fut: Future<Output = Result<V, E>>>(
        self,
        f: F,
    ) -> impl Future<Output = Result<V, E>>
    where
        CommonError: Into<E>;
}

impl RequestBuilderExt for RequestBuilder {
    fn disable_cookie(self) -> RequestBuilder {
        self.header(header::COOKIE, "")
    }
    async fn then<V, E, F: FnOnce(Response) -> Fut, Fut: Future<Output = Result<V, E>>>(
        self,
        res: F,
    ) -> Result<V, E>
    where
        CommonError: Into<E>,
    {
        let result = self.send().await;
        match result {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(CommonError::Response(response.status().as_u16()).into());
                }
                res(response).await
            }
            Err(e) => Err(CommonError::Network(e).into()),
        }
    }
}
