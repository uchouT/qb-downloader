// TODO: test qb and uploader is ok
//! check if is authenticated

use super::{Action, BoxBody, Req, ServerResult};
use crate::server::ResultResponse;
use hyper::Response;

#[derive(Default, Debug)]
pub struct TestAPI;
impl Action for TestAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        match *req.method() {
            _ => Ok(ResultResponse::success()),
        }
    }
}
