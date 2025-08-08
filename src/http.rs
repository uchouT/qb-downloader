use crate::error::Error;
use log::error;
use reqwest::{Client, RequestBuilder, Response, cookie::Jar, header};
use std::{
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
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
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

pub trait Extra {
    fn disable_cookie(self) -> RequestBuilder;
    fn then<V, F: FnOnce(Response) -> Fut, Fut: Future<Output = Result<V, Error>>>(
        self,
        f: F,
    ) -> impl Future<Output = Result<V, Error>>;
}

impl Extra for RequestBuilder {
    fn disable_cookie(self) -> RequestBuilder {
        self.header(header::COOKIE, "")
    }
    fn then<V, F: FnOnce(Response) -> Fut, Fut: Future<Output = Result<V, Error>>>(
        self,
        res: F,
    ) -> impl Future<Output = Result<V, Error>> {
        async move {
            let result = self.send().await;
            match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        let msg = format!(
                            "Qbittorrent error: {} \n{}",
                            response.status(),
                            response.text().await.unwrap_or_default()
                        );
                        error!("{}", msg);
                        return Err(Error::Qb(msg));
                    }
                    res(response).await
                }
                Err(e) => {
                    let msg = format!("Network error: {}", e);
                    error!("{}", msg);
                    Err(Error::Network(msg))
                }
            }
        }
    }
}
