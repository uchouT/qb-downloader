//! Login api

use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    auth::{Login, TOKEN, encode},
    config::{self, Account},
    server::{ResultResponse, api::from_json_owned},
};
use hyper::Response;
use rand::{Rng, distr::Alphanumeric};

#[derive(Default, Debug)]
pub struct LoginAPI;
impl Action for LoginAPI {
    fn needs_auth(&self) -> bool {
        false
    }
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        let account: Account = from_json_owned(req).await?;

        let general_cfg = config::value().general().await;
        if general_cfg.account.password != encode(&account.password)
            || general_cfg.account.username != account.username
        {
            return Ok(ResultResponse::error_msg("invalid username or password"));
        }
        let key = if general_cfg.multi_login {
            ""
        } else {
            &gen_key(32)
        };
        let login = Login {
            account: &general_cfg.account,
            key,
        };
        let new_token = encode(&serde_json::to_string(&login).unwrap_or_default());
        *TOKEN.get().unwrap().write().unwrap() = new_token.clone();
        Ok(ResultResponse::success_data(new_token))
    }
}

pub fn gen_key(size: usize) -> String {
    let mut rng = rand::rng();
    (0..size)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
