pub mod api;
pub mod auth;
pub mod error;
use crate::{
    config,
    errors::{CommonError, TargetContextedResult, format_error_chain},
};

use api::Action;
use error::handle;
use futures_util::{FutureExt, select};
use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    {self, Request, Response, StatusCode, header},
};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, warn};
use serde::Serialize;
use std::{
    borrow::Cow,
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpListener, sync::broadcast, time::sleep};
type BoxBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;
type ServerResult<T> = std::result::Result<T, Infallible>;
type Req = Request<Incoming>;

macro_rules! define_routes {
    ($($path: literal => $action_type: ty), * $(,)?) => {
        async fn route(req: Req, socket_addr: SocketAddr) -> ServerResult<Response<BoxBody>> {
            if config::value().general.is_only_inner_ip && !is_inner_ip(socket_addr) {
                return Ok(ResultResponse::error_with_code(StatusCode::FORBIDDEN));
            }
            match req.uri().path() {
                $(
                    $path => {
                        let action =  <$action_type>::default();
                        if action.needs_auth() {
                            if let Err(e) = action.auth(&req) {
                                return handle(e);
                            }
                        }
                        match action.execute(req).await {
                            Ok(res) => Ok(res),
                            Err(e) => handle(e),
                        }
                    }
                )*
                _ => match api::asset_api::AssetAPI.execute(req).await {
                    Ok(res) => Ok(res),
                    Err(e) => handle(e),
                }
            }
        }
    };
}

define_routes! {
    "/api/config" => api::config_api::ConfigAPI,
    "/api/task" => api::task_api::TaskAPI,
    "/api/torrent" => api::torrent_api::TorrentAPI,
    "/api/login" => api::login_api::LoginAPI,
    "/api/test" => api::test_api::TestAPI,
    "/api/version" => api::version_api::VersionAPI,
}

pub async fn run(
    mut shutdown_rx: broadcast::Receiver<()>,
    addr: Option<IpAddr>,
    port: u16,
) -> std::result::Result<(), CommonError> {
    let addr: SocketAddr = if let Some(ip) = addr {
        (ip, port).into()
    } else {
        ([127, 0, 0, 1], port).into()
    };
    let listener = TcpListener::bind(addr)
        .await
        .convert_then_add_context("Failed to bind to address")?;
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    info!("Listening on http://{addr}");

    loop {
        select! {
            accept_result = listener.accept().fuse() => {
                match accept_result {
                    Ok((stream, socket_addr)) => {
                        debug!("Accepted connection from {socket_addr}");

                        let io = TokioIo::new(stream);
                        let conn =  http1::Builder::new().serve_connection(io, service_fn(move |req| {
                            route(req, socket_addr)
                        }));
                        let fut = graceful.watch(conn);
                        tokio::spawn(async move {
                            if let Err(e) = fut.await {
                                error!("Error serving connection {socket_addr}: {}", format_error_chain(e));
                            } else {
                                debug!("Connection from {socket_addr} closed");
                            }
                        });
                    }

                    Err(e) => {
                        error!("Failed to accept connection: {}", format_error_chain(e));
                        continue;
                    }
                }
            }

            _ = shutdown_rx.recv().fuse() => {
                drop(listener);
                info!("Server received shutdown signal");
                break;
            }
        }
    }
    info!("Server is shutting down");
    select! {
        _ = graceful.shutdown().fuse() => {
            info!("Server has shut down gracefully");
        }
        _ = sleep(Duration::from_secs(5)).fuse() => {
            warn!("Server shutdown timed out, continuing...");
        }
    }
    Ok(())
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn empty() -> BoxBody {
    Empty::new().map_err(|never| match never {}).boxed()
}

fn is_inner_ip(socket_addr: SocketAddr) -> bool {
    match socket_addr.ip() {
        IpAddr::V4(addr) => addr.is_loopback() || addr.is_private(),
        IpAddr::V6(addr) => {
            addr.is_loopback() || addr.is_unique_local() || addr.is_unicast_link_local()
        }
    }
}
/// RESTful API response data structure
#[derive(Debug, Serialize)]
struct ResultResponse<T: Serialize> {
    /// Response message which will show in the frontend
    pub message: Option<Cow<'static, str>>,

    pub code: u16,
    /// Contain the actual response data
    pub data: Option<T>,
}

impl ResultResponse<()> {
    /// Create a successful response without data
    fn success() -> Response<BoxBody> {
        let result = Self {
            message: Some("Success".into()),
            data: None,
            code: StatusCode::OK.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }

    /// Create a successful response with a custom message
    fn success_msg(msg: impl Into<Cow<'static, str>>) -> Response<BoxBody> {
        let result = Self {
            message: Some(msg.into()),
            data: None,
            code: StatusCode::OK.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }

    // /// Create a error response
    // fn error() -> Response<BoxBody> {
    //     let result = Self {
    //         message: Some("Error".into()),
    //         data: None,
    //         code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
    //     };
    //     let json = serde_json::to_string(&result).unwrap();
    //     Response::builder()
    //         .status(StatusCode::INTERNAL_SERVER_ERROR)
    //         .header(header::CONTENT_TYPE, "application/json")
    //         .body(full(json))
    //         .unwrap()
    // }

    /// Create a error response with message
    fn error_msg(msg: impl Into<Cow<'static, str>>) -> Response<BoxBody> {
        let result = Self {
            message: Some(msg.into()),
            data: None,
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }

    fn unauthorized() -> Response<BoxBody> {
        let result = Self {
            message: Some("unauthorized".into()),
            data: None,
            code: StatusCode::FORBIDDEN.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }

    /// build a bad request response, with optional message
    fn bad_request(message: Option<Cow<'static, str>>) -> Response<BoxBody> {
        let result = Self {
            message,
            data: None,
            code: StatusCode::BAD_REQUEST.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }

    fn error_with_code(code: StatusCode) -> Response<BoxBody> {
        let mut res = Response::new(empty());
        *res.status_mut() = code;
        res
    }
}

impl<T: Serialize> ResultResponse<T> {
    /// Create a successful response with data
    fn success_data(data: T) -> Response<BoxBody> {
        let result = Self {
            message: Some("Success".into()),
            data: Some(data),
            code: StatusCode::OK.as_u16(),
        };
        let json = serde_json::to_string(&result).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap()
    }
}
