//!  end point at "/api/config"
use super::{Action, BoxBody, Req, ServerResult};
use crate::{
    auth::{Login, TOKEN, encode},
    config::{self, Account, ConfigValue},
    error::CommonError,
    qb,
    server::{
        ResultResponse,
        api::{from_json_owned, login_api::gen_key},
        error::{ServerError, ServerErrorKind},
    },
};
use futures_util::future::join;
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
async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let mut config: ConfigValue = from_json_owned(req).await?;
    let account_bak = config::value().general.account.clone();

    // account info has changed
    let account_changed =
        !config.general.account.username.is_empty() || !config.general.account.password.is_empty();
    if account_changed {
        config.general.account.username = if config.general.account.username.is_empty() {
            account_bak.username
        } else {
            config.general.account.username
        };
        config.general.account.password = if config.general.account.password.is_empty() {
            account_bak.password
        } else {
            encode(&config.general.account.password)
        }
    } else {
        config.general.account = account_bak;
    }

    let (_, config_res) = join(
        qb::login_with(
            &config.qb.qb_host.clone(),
            &config.qb.qb_username.clone(),
            &config.qb.qb_password.clone(),
        ),
        update_config(config, account_changed),
    )
    .await;
    if let Err(e) = config_res {
        error!("error saving config: {e}");
    }
    Ok(ResultResponse::success_msg(
        "Configuration updated successfully",
    ))
}

/// update config with config value
async fn update_config(config: ConfigValue, account_changed: bool) -> Result<(), CommonError> {
    config::set_value(config);
    let config = config::value();

    // refresh token if account changed
    if account_changed {
        let general_cfg = &config.general;
        let key = if general_cfg.multi_login {
            ""
        } else {
            &gen_key(32)
        };
        let login = Login {
            key,
            account: &general_cfg.account,
        };
        *TOKEN.get().unwrap().write().unwrap() =
            encode(&serde_json::to_string(&login).unwrap_or_default());
    }
    config::save().await
}

/// Get config
async fn get() -> ServerResult<Response<BoxBody>> {
    let mut config = config::value().as_ref().clone();
    config.general.account = Account {
        username: String::new(),
        password: String::new(),
    };

    Ok(ResultResponse::success_data(config))
}
