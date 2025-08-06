use crate::{
    Error, config, format_error_chain,
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
    NEW,

    // new added torrent, but haven't added to task list yet.
    WAITED,
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        match self {
            Tag::NEW => "qbd_new",
            Tag::WAITED => "qbd_waited",
        }
    }
}

#[derive(Debug)]
pub struct Qb {
    host: String,
    logined: bool,
    // TODO: add vertion
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

pub fn init() -> Result<(), Error> {
    let _ = QB
        .set(RwLock::new(Qb::new()))
        .expect("Failed to initialize qBittorrent client");
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

fn get_host() -> Result<String, Error> {
    let qb = QB.get().unwrap().read().map_err(|e| {
        let poison_error = Error::Qb(e.to_string().into());
        error!(
            "Failed to read qBittorrent client: {}",
            format_error_chain(&poison_error)
        );
        panic!();
    })?;
    if qb.logined {
        Ok(qb.host.clone())
    } else {
        error!("qBittorrent access denied");
        Err(Error::Qb("Not logged in to qBittorrent".into()))
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
pub async fn get_torrent_info() -> Result<Vec<TorrentInfo>, Error> {
    let host = get_host()?;
    let param = HashMap::from([("category", CATEGORY)]);
    let result = http::get(format!("{}/api/v2/torrents/info", host))
        .form(&param)
        .send()
        .await;
    match result {
        Ok(res) => {
            if res.status() != 200 {
                return Err(Error::Qb("Failed to get torrent info".into()));
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
        Err(e) => Err(Error::Qb(e.to_string().into())),
    }
}

/// get the new added torrent hash, and remove teh marked tag
pub async fn get_hash() -> Result<String, Error> {
    let host = get_host()?;
    let mut param = HashMap::new();
    param.insert("category", CATEGORY);
    param.insert("tag", Tag::NEW.as_ref());

    let result = http::get(format!("{}/api/v2/torrents/info", host))
        .form(&param)
        .send()
        .await;
    match result {
        Ok(res) => {
            if res.status().is_success() {
                return Err(Error::Qb("Failed to get torrent info".into()));
            }
            let json: Value = res.json().await.unwrap();
            let hash = json.get("hash").and_then(|v| v.as_str()).unwrap();
            remove_tag(hash, Tag::NEW).await?;
            Ok(hash.to_string())
        }
        Err(_) => Err(Error::Qb("Failed to get torrent info".into())),
    }
}

async fn manage_tag(hash: &str, tag: Tag, action: &str) -> Result<(), Error> {
    let host = get_host()?;
    let param = HashMap::from([("hashes", hash), ("tags", tag.as_ref())]);
    let result = http::post(format!("{}/api/v2/torrents/{}", host, action))
        .form(&param)
        .send()
        .await;
    match result {
        Ok(res) => {
            if res.status().is_success() {
                return Ok(());
            }
            Err(Error::Qb(res.text().await.unwrap_or_default().into()))
        }
        Err(e) => Err(Error::Network(e.to_string())),
    }
}

/// remove the tag of the corresponding torrent
pub async fn remove_tag(hash: &str, tag: Tag) -> Result<(), Error> {
    manage_tag(hash, tag, "removeTags").await
}

/// add tag to the corresponding torrent
pub async fn add_tag(hash: &str, tag: Tag) -> Result<(), Error> {
    manage_tag(hash, tag, "addTags").await
}

/// manage torrent task
async fn manage(hash: &str, action: &str) -> Result<(), Error> {
    let host = get_host()?;
    let result = http::post(format!("{}/api/v2/torrents/{}", host, action))
        .form(&HashMap::from([("hashes", hash)]))
        .send()
        .await;
    match result {
        Ok(res) => {
            if res.status().is_success() {
                return Ok(());
            }
            Err(Error::Qb(res.text().await.unwrap_or_default().into()))
        }
        Err(e) => Err(Error::Network(e.to_string())),
    }
}

/// start a torrent
pub async fn start(hash: &str) -> Result<(), Error> {
    manage(hash, "start").await
}

/// stop a torrent
pub async fn stop(hash: &str) -> Result<(), Error> {
    manage(hash, "stop").await
}
