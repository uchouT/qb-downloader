//! endpoint at "/api/test"
//! check if is authenticated
//! # POST
//! check if the provided test item is valid
//! # GET
//! check if qbittorrent is logged in
//! Else return success to test authentication
use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    qb,
    server::{
        ResultResponse,
        api::{from_json, get_json_body},
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

    let valid = match test_req.test_type {
        "qb" => qb::test_login(test_req.host, test_req.username, test_req.password).await,
        "Rclone" => Rclone::test(test_req.host, test_req.username, test_req.password).await,
        _ => return Ok(ResultResponse::bad_request(Some("unknown test type"))),
    };
    if valid {
        Ok(ResultResponse::success())
    } else {
        Ok(ResultResponse::error_msg("test failed"))
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
    let qb_ok = qb::is_logined().await;
    Ok(ResultResponse::success_data(qb_ok))
}
