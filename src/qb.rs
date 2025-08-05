use crate::{
    config,
    http::{self, DisableCookie},
    remove_slash,
};
use log::error;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

const CATEGORY: &str = "QBD";

/// qBittorrent tag
pub enum Tag {
    // new added torrent, but haven't fetched meta data yet.
    NEW(String),

    // new added torrent, but haven't added to task list yet.
    WAITED(String),
}
pub struct Qb {
    host: String,
    logined: bool,
}

#[derive(Clone)]
pub struct TorrentInfo {
    pub hash: String,
    pub state: String,
    pub progress: f64,
}

impl TorrentInfo {
    pub fn new(hash: String, state: String, progress: f64) -> Self {
        TorrentInfo {
            hash,
            state,
            progress,
        }
    }
}
static QB: OnceLock<RwLock<Qb>> = OnceLock::new();
// TODO: error and log design
pub fn init() -> Result<(), String> {
    let _ = QB.set(RwLock::new(Qb::new()));
    Ok(())
}
impl Qb {
    pub fn new() -> Self {
        Qb {
            host: config::read(|c| c.qb_host.clone()),
            logined: false,
        }
    }
}

/// login to qBittorrent, and update the host and logined status
pub async fn login() -> bool {
    let (host, username, pass) = config::read(|c| {
        (
            c.qb_host.clone(),
            c.qb_username.clone(),
            c.qb_password.clone(),
        )
    });
    let refined_host = remove_slash(host);
    let logined = test_login(&refined_host.as_str(), username.as_str(), pass.as_str()).await;
    if let Ok(mut qb) = QB.get().unwrap().write() {
        qb.host = refined_host;
        qb.logined = logined;
    }
    logined
}

pub async fn test_login(host: &str, username: &str, pass: &str) -> bool {
    let refined_host = remove_slash(&host);
    let mut form = HashMap::new();
    form.insert("username", username);
    form.insert("password", pass);
    let result = http::post(format!("{}/api/v2/auth/login", refined_host).as_str())
        .form(&form)
        .disable_cookie()
        .send()
        .await;
    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// get to torrent info list, with CATEGORY
pub async fn get_torrent_info() -> Result<Vec<TorrentInfo>, String> {
    let host;
    if let Ok(qb) = QB.get().unwrap().read() {
        if !qb.logined {
            let msg = "Not logged in to qBittorrent";
            error!("{msg}");
            return Err(msg.to_string());
        }
        host = qb.host.clone();
    } else {
        return Err("".to_string());
    }
    let param = HashMap::from([("category", CATEGORY)]);
    let result = http::get(format!("{}/api/v2/torrents/info", host))
        .form(&param)
        .send()
        .await;
    match result {
        Ok(res) => {
            if res.status() != 200 {
                return Err("".to_string());
            }
            let json: Value = res.json().await.unwrap();
            let mut torrent_info_list = Vec::new();
            for item in json.as_array().unwrap().iter() {
                let state = item.get("state").and_then(|v| v.as_str()).unwrap();
                let hash = item.get("hash").and_then(|v| v.as_str()).unwrap();
                let progress = item.get("progress").and_then(|v| v.as_f64()).unwrap();
                torrent_info_list.push(TorrentInfo::new(
                    hash.to_string(),
                    state.to_string(),
                    progress,
                ));
            }
            Ok(torrent_info_list)
        }
        Err(e) => Err("".to_string()),
    }
}