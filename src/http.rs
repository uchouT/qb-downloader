use reqwest::{Client, RequestBuilder, cookie::Jar};
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

pub fn post(url: &str) -> RequestBuilder {
    get_client().post(url)
}

pub fn get(url: &str) -> RequestBuilder {
    get_client().get(url)
}

pub fn delete(url: &str) -> RequestBuilder {
    get_client().delete(url)
}

pub fn put(url: &str) -> RequestBuilder {
    get_client().put(url)
}
