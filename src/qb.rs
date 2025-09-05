//! This module provides API to interact with qBittorrent

pub mod error;
use crate::{
    config, remove_slash,
    request::{self, RequestBuilderExt},
};
use base32::Alphabet;
use error::{QbError, QbErrorKind};
use log::{error, info, warn};
use reqwest::multipart;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};
use tokio::time::sleep;
const CATEGORY: &str = "QBD";

/// qBittorrent tag
pub enum Tag {
    // new added torrent, but haven't fetched meta data yet.
    New,

    // new added torrent, but haven't added to task list yet.
    Waited,
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        match self {
            Tag::New => "qbd_new",
            Tag::Waited => "qbd_waited",
        }
    }
}

#[derive(Debug)]
pub struct Qb {
    host: Arc<str>,
    logined: bool,
    version: u8,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TorrentInfo {
    pub hash: String,
    pub state: String,
    pub progress: f64,
}

static QB: OnceLock<RwLock<Qb>> = OnceLock::new();

pub async fn init() {
    QB.set(RwLock::new(Qb::new().await))
        .expect("Failed to initialize qBittorrent client");
}
impl Qb {
    pub async fn new() -> Self {
        Qb {
            host: Arc::from(config::value().qb().await.qb_host.as_str()),
            logined: false,
            version: 0,
        }
    }
}
pub fn is_logined() -> bool {
    QB.get().unwrap().read().unwrap().logined
}

/// get the host if logged in, else return error
fn host() -> Result<Arc<str>, QbError> {
    let qb = QB.get().unwrap().read().unwrap();
    if qb.logined {
        Ok(qb.host.clone())
    } else {
        error!("qBittorrent access denied");
        Err(QbError {
            kind: QbErrorKind::NotLogin,
        })
    }
}

fn version() -> u8 {
    QB.get().unwrap().read().unwrap().version
}

/// try to login with qb info in config
pub async fn login() {
    let qb_cfg = config::value().qb().await;
    login_with(&qb_cfg.qb_host, &qb_cfg.qb_username, &qb_cfg.qb_password).await;
}

/// login to qBittorrent, and update the host and logined status
/// # Precondition
/// - host has been normalized.
pub async fn login_with(host: &str, username: &str, password: &str) {
    let logined = test_login(host, username, password).await;
    QB.get().unwrap().write().unwrap().host = Arc::from(host);
    if logined {
        match get_version(host).await {
            Ok(v) => {
                let mut qb = QB.get().unwrap().write().unwrap();
                qb.version = v;
                qb.logined = true;
                info!("qBittorrent login successful");
            }
            Err(e) => {
                if matches!(e.kind, QbErrorKind::UnsupportedVersion) {
                    warn!("qBittorrent version is not supported");
                } else {
                    error!("Failed to get qBittorrent version: {e}");
                }
                let mut qb = QB.get().unwrap().write().unwrap();
                qb.logined = false;
                qb.version = 0;
            }
        }
    } else {
        let mut qb = QB.get().unwrap().write().unwrap();
        qb.logined = false;
        warn!("qBittorrent login failed");
    }
}

async fn get_version(host: &str) -> Result<u8, QbError> {
    request::get(format!("{host}/api/v2/app/version"))
        .then(async |res| {
            let ver = res.text().await?;
            let c = ver
                .strip_prefix("v")
                .and_then(|v| Some(v.split('.').collect::<Vec<&str>>()))
                .unwrap();
            let first = c[0].parse::<u8>().unwrap();
            if first == 5 {
                return Ok(first);
            } else if first == 4 {
                let second = c[1].parse::<u8>().unwrap();
                if second >= 1 {
                    return Ok(first);
                }
            }
            Err(QbError {
                kind: QbErrorKind::UnsupportedVersion,
            })
        })
        .await
}

/// accept any form of host, e.g. "http://example.com" or "example.com"
pub async fn test_login(host: &str, username: &str, pass: &str) -> bool {
    let refined_host = remove_slash(host);
    let mut form = HashMap::new();
    form.insert("username", username);
    form.insert("password", pass);
    let result = request::post(format!("{refined_host}/api/v2/auth/login").as_str())
        .form(&form)
        .disable_cookie()
        .send()
        .await;

    if let Ok(res) = result {
        res.text().await.unwrap_or_default() == "Ok."
    } else {
        false
    }
}

/// get all torrent infos with CATEGORY
pub async fn get_torrent_info() -> Result<Vec<TorrentInfo>, QbError> {
    let host = host()?;
    request::get(format!("{host}/api/v2/torrents/info"))
        .query(&[("category", CATEGORY)])
        .then(async |res| {
            let torrent_info_list: Vec<TorrentInfo> = res.json().await?;
            Ok(torrent_info_list)
        })
        .await
}

/// get the new added torrent hash, according to the tag [`Tag::New`]
pub async fn get_hash() -> Result<String, QbError> {
    let host = host()?;
    request::get(format!("{host}/api/v2/torrents/info"))
        .query(&[("category", CATEGORY), ("tag", Tag::New.as_ref())])
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            // Always occurs when a same torrent is added
            if json_array.is_empty() {
                return Err(QbError {
                    kind: QbErrorKind::NoNewTorrents,
                });
            }
            let hash = json_array[0].get("hash").and_then(|v| v.as_str()).unwrap();
            Ok(hash.to_string())
        })
        .await
}

async fn manage_tag(hash: &str, tag: Tag, action: &str) -> Result<(), QbError> {
    let host = host()?;
    let param = HashMap::from([("hashes", hash), ("tags", tag.as_ref())]);
    request::post(format!("{host}/api/v2/torrents/{action}"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// remove the tag of the corresponding torrent
pub async fn remove_tag(hash: &str, tag: Tag) -> Result<(), QbError> {
    manage_tag(hash, tag, "removeTags").await
}

/// add tag to the corresponding torrent
pub async fn add_tag(hash: &str, tag: Tag) -> Result<(), QbError> {
    manage_tag(hash, tag, "addTags").await
}

/// manage torrent task
async fn manage(hash: &str, action: &str) -> Result<(), QbError> {
    let host = host()?;
    request::post(format!("{host}/api/v2/torrents/{action}"))
        .form(&[("hashes", hash)])
        .then(async |_| Ok(()))
        .await
}

/// start a torrent
pub async fn start(hash: &str) -> Result<(), QbError> {
    if version() < 5 {
        manage(hash, "resume").await
    } else {
        manage(hash, "start").await
    }
}

/// stop a torrent
pub async fn stop(hash: &str) -> Result<(), QbError> {
    if version() < 5 {
        manage(hash, "pause").await
    } else {
        manage(hash, "stop").await
    }
}

/// delete a torrent
pub async fn delete(hash: &str, delete_files: bool) -> Result<(), QbError> {
    let host = host()?;
    let param = HashMap::from([
        ("hashes", hash),
        ("deleteFiles", if delete_files { "true" } else { "false" }),
    ]);
    request::post(format!("{host}/api/v2/torrents/delete"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// get the state of a torrent
pub async fn get_state(hash: &str) -> Result<String, QbError> {
    let host = host()?;
    request::get(format!("{host}/api/v2/torrents/info"))
        .query(&[("hashes", hash)])
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            if json_array.is_empty() {
                return Err(QbError {
                    kind: QbErrorKind::NoNewTorrents,
                });
            }
            let state = json_array[0].get("state").and_then(|v| v.as_str()).unwrap();
            Ok(state.to_string())
        })
        .await
}

/// set the download priority of a torrent
/// # Arguments
/// * `hash` - The hash of the torrent.
/// * `priority` - 1 for download, 0 for not download.
/// * `index_list` - the index list of files need to set.
pub async fn set_prio(hash: &str, priority: u8, index_list: &[usize]) -> Result<(), QbError> {
    let host = host()?;
    let id = index_list
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("|");

    let prio = priority.to_string();
    let param = HashMap::from([
        ("hash", hash),
        ("priority", prio.as_str()),
        ("id", id.as_str()),
    ]);
    request::post(format!("{host}/api/v2/torrents/filePrio"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// set the task not download, usually used when a new part of task is added
pub async fn set_not_download(hash: &str, file_num: usize) -> Result<(), QbError> {
    set_prio(hash, 0, (0..file_num).collect::<Vec<usize>>().as_slice()).await
}

/// set the share limit of a torrent
pub async fn set_share_limit(
    hash: &str,
    ratio_limit: f64,
    seeding_time_limit: i32,
) -> Result<(), QbError> {
    let host = host()?;
    let param = HashMap::from([
        ("hashes", hash.to_string()),
        ("ratioLimit", ratio_limit.to_string()),
        ("seedingTimeLimit", seeding_time_limit.to_string()),
        ("inactiveSeedingTimeLimit", "-2".to_string()),
    ]);
    request::post(format!("{host}/api/v2/torrents/setShareLimits"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// export .torrent file to a specified path, and remove the [`Tag::New`]
pub async fn export(hash: &str, path: &Path) -> Result<(), QbError> {
    // wait for the torrent to fetch meta data
    loop {
        let state = get_state(hash).await?;
        if ["stoppedUP", "pausedUP", "stoppedDL", "pausedDL"].contains(&state.as_str()) {
            break;
        }
        sleep(std::time::Duration::from_secs(1)).await;
    }
    let host = host()?;
    request::post(format!("{host}/api/v2/torrents/export"))
        .form(&[("hash", hash)])
        .then(async |res| {
            let data = res.bytes().await?;
            let mut file = File::create(path)?;
            file.write_all(&data)?;
            remove_tag(hash, Tag::New).await?;
            Ok(())
        })
        .await
}

/// get the hash list of torrents with a specific tag
pub async fn get_tag_torrent_list(tag: Tag) -> Result<Vec<String>, QbError> {
    let host = host()?;
    request::get(format!("{host}/api/v2/torrents/info"))
        .query(&[("category", CATEGORY), ("tag", tag.as_ref())])
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            let hash_list = json_array
                .iter()
                .filter_map(|v| v.get("hash").and_then(|h| h.as_str().map(String::from)))
                .collect::<Vec<String>>();
            Ok(hash_list)
        })
        .await
}

/// add a torrent to qBittorrent by URL
/// if url is a magnet link, means hash is known, else add tag New and wait for [`get_hash`] to fetch the meta data
pub async fn add_by_url(url: &str, save_path: &str) -> Result<(), QbError> {
    let host = host()?;
    let param = HashMap::from([
        ("urls", url),
        ("savepath", save_path),
        ("category", CATEGORY),
        ("stopCondition", "MetadataReceived"),
        ("tags", Tag::New.as_ref()),
    ]);

    request::post(format!("{host}/api/v2/torrents/add"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// add a torrent to qBittorrent by file path,
/// aiming to fast recover from cached .torrrent file
pub async fn add_by_file(
    torrent_path: &Path,
    save_path: &str,
    seeding_time_limit: i32,
    ratio_limit: f64,
) -> Result<(), QbError> {
    let host = host()?;
    let file_part = multipart::Part::file(torrent_path).await?;
    let form = multipart::Form::new()
        .part("torrents", file_part)
        .text("category", CATEGORY)
        .text("savepath", save_path.to_string())
        .text("seedingTimeLimit", seeding_time_limit.to_string())
        .text("ratioLimit", ratio_limit.to_string())
        .text("stopped", "true");
    request::post(format!("{host}/api/v2/torrents/add"))
        .multipart(form)
        .then(async |_| Ok(()))
        .await
}

/// add a torrent to qBittorrent by bytes
pub async fn add_by_bytes(file_name: &str, save_path: &str, data: &[u8]) -> Result<(), QbError> {
    let host = host()?;
    let file_part = multipart::Part::bytes(data.to_vec()).file_name(file_name.to_string());
    let form = multipart::Form::new()
        .part("torrents", file_part)
        .text("savepath", save_path.to_string())
        .text("category", CATEGORY)
        .text("stopped", "true");
    request::post(format!("{host}/api/v2/torrents/add"))
        .multipart(form)
        .then(async |_| Ok(()))
        .await
}

/// Try to parse the hash from a url first, usually used to parse magnet link
/// If failed, it means the url is probably an http link, e.g. "http://example.com/file.torrent"
pub fn try_parse_hash(url: &str) -> Option<String> {
    if let Some(mut hash) = url.strip_prefix("magnet:?xt=urn:btih:") {
        if let Some(end) = hash.find('&') {
            hash = &hash[..end];
        }
        let mut hash = hash.to_string();
        // Base32
        if hash.len() == 32
            && let Some(hash_raw) = base32::decode(Alphabet::Rfc4648 { padding: false }, &hash)
        {
            hash = hash_raw
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
        } else {
            // parse base32 failed
            return None;
        }
        Some(hash)
    } else {
        None
    }
}
