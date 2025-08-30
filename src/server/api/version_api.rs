//! endpoint at "/api/version"
use super::{Action, BoxBody, Req, ServerResult};
use crate::{VERSION, server::ResultResponse};
use hyper::Response;

#[derive(Default, Debug)]
pub struct VersionAPI;
impl Action for VersionAPI {
    async fn execute(&self, _: Req) -> ServerResult<Response<BoxBody>> {
        Ok(ResultResponse::success_data(VERSION))
    }
    fn needs_auth(&self) -> bool {
        false
    }
}
