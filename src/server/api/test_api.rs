//! endpoint at "/api/test"
//! check if is authenticated
//! # POST
//! check if the provided test item is valid
//! # GET
//! check if qbittorrent is logged in
//! Else return success to test authentication
use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    error::QbError,
    qb,
    server::{
        ResultResponse,
        api::{from_json, get_json_body},
        error::ServerError,
    },
    upload::{Rclone, UploaderTrait},
};
use hyper::{Method, Response};
use serde::Deserialize;

#[derive(Default, Debug)]
pub struct TestAPI;
impl Action for TestAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        match *req.method() {
            Method::POST => post(req).await,
            Method::GET => get().await,
            _ => Ok(ResultResponse::success()),
        }
    }
}

async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let data = get_json_body(req).await?;
    let test_req: TestReq = from_json(&data)?;

    match test_req.test_type {
        "qb" => {
            if !qb::test_login(test_req.host, test_req.username, test_req.password, true).await {
                return Ok(ResultResponse::error_msg("Qbittorrent failed to login"));
            }
            if let Err(e) = qb::get_version(test_req.host).await {
                if let QbError::UnsupportedVersion = e {
                    return Ok(ResultResponse::error_msg("Unsupported qbittorrent version"));
                } else {
                    return Err(ServerError::Unknown(
                        anyhow::Error::from(e).context("Failed to get qbittorrent version"),
                    ));
                }
            }
            Ok(ResultResponse::success())
        }
        "Rclone" => {
            if Rclone::test(test_req.host, test_req.username, test_req.password).await {
                Ok(ResultResponse::success())
            } else {
                Ok(ResultResponse::error_msg("Rclone test failed"))
            }
        }
        _ => Ok(ResultResponse::bad_request(Some("unknown test type"))),
    }
}

#[derive(Deserialize)]
struct TestReq<'a> {
    #[serde(borrow)]
    test_type: &'a str,
    #[serde(borrow)]
    host: &'a str,
    #[serde(borrow)]
    username: &'a str,
    #[serde(borrow)]
    password: &'a str,
}

async fn get() -> ServerResult<Response<BoxBody>> {
    let qb_ok = qb::is_logined();
    Ok(ResultResponse::success_data(qb_ok))
}
