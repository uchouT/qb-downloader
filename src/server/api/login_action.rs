//! Login api

use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    Entity,
    auth::{Login, TOKEN, encode},
    config::{Account, Config},
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
        Config::read(|c| {
            if c.account.password != encode(&account.password)
                || c.account.username != account.username
            {
                return Ok(ResultResponse::error_msg("invalid username or password"));
            }
            let key = if c.multi_login { "" } else { &gen_key(32) };
            let login = Login {
                account: &c.account,
                key: &key,
            };
            let new_token = encode(&serde_json::to_string(&login).unwrap_or_default());
            *TOKEN.get().unwrap().write().unwrap() = new_token.clone();
            Ok(ResultResponse::success_data(new_token))
        })
        .await
    }
}

pub fn gen_key(size: usize) -> String {
    let mut rng = rand::rng();
    (0..size)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
