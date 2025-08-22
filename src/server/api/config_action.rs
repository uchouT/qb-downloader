//!  end point at "/api/config"
use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    Entity,
    auth::{Login, TOKEN, encode},
    config::{Account, Config, ConfigValue},
    error::CommonError,
    qb,
    server::{
        ResultResponse,
        api::{from_json_owned, login_action::gen_key},
        error::{ServerError, ServerErrorKind},
    },
};
use futures::future::join;
use hyper::{Method, Response};
use log::error;

#[derive(Default)]
pub struct ConfigAPI;
impl Action for ConfigAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        match *req.method() {
            Method::POST => post(req).await,
            Method::GET => get().await,
            _ => Err(ServerError {
                kind: ServerErrorKind::MethodNotAllowed,
            }),
        }
    }
}

/// Change config from post
/// TODO: test
async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let mut config: ConfigValue = from_json_owned(req).await?;
    let account_bak = Config::read(|c| c.account.clone()).await;

    // account info has changed
    let account_changed =
        !config.account.username.is_empty() || !config.account.password.is_empty();
    if account_changed {
        config.account.username = if config.account.username.is_empty() {
            account_bak.username
        } else {
            config.account.username
        };
        config.account.password = if config.account.password.is_empty() {
            account_bak.password
        } else {
            encode(&config.account.password)
        }
    } else {
        config.account = account_bak;
    }

    let (config_res, _) = join(update_config(config, account_changed), qb::login()).await;
    if let Err(e) = config_res {
        error!("error saving config: {e}");
    }
    Ok(ResultResponse::success_msg(
        "Configuration updated successfully",
    ))
}

async fn update_config(config: ConfigValue, account_changed: bool) -> Result<(), CommonError> {
    Config::write(|c| {
        if account_changed {
            // refresh token
            let key = if c.multi_login { "" } else { &gen_key(32) };
            let login = Login {
                account: &c.account,
                key: &key,
            };
            *TOKEN.get().unwrap().write().unwrap() =
                encode(&serde_json::to_string(&login).unwrap_or_default());
        }
        *c = config;
    })
    .await;
    Config::save().await
}

/// Get config
async fn get() -> ServerResult<Response<BoxBody>> {
    let config = ConfigValue {
        account: Account {
            username: String::new(),
            password: String::new(),
        },
        ..Config::read(|c| c.clone()).await
    };

    Ok(ResultResponse::success_data(config))
}
