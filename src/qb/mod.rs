//! This module provides API to interact with qBittorrent

pub mod error;
use crate::{
    Entity,
    config::Config,
    http::{self, Extra},
    remove_slash,
    task::TaskValue,
};
use error::{QbError, QbErrorKind};
use log::error;
use reqwest::multipart;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::Write, sync::OnceLock};
use tokio::{sync::RwLock, time::sleep};
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

#[derive(Clone, Deserialize, Debug)]
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

pub async fn init() -> Result<(), QbError> {
    QB.set(RwLock::new(Qb::new().await))
        .expect("Failed to initialize qBittorrent client");
    Ok(())
}
impl Qb {
    pub async fn new() -> Self {
        Qb {
            host: Config::read(|c| c.qb_host.clone()).await,
            logined: false,
        }
    }
}
pub async fn is_logined() -> bool {
    QB.get().unwrap().read().await.logined
}
async fn get_host() -> Result<String, QbError> {
    let qb = QB.get().unwrap().read().await;
    if qb.logined {
        Ok(qb.host.clone())
    } else {
        error!("qBittorrent access denied");
        Err(QbError {
            kind: QbErrorKind::NotLogin,
        })
    }
}
/// login to qBittorrent, and update the host and logined status
pub async fn login()  {
    let (host, username, pass) = Config::read(|c| {
        (
            c.qb_host.clone(),
            c.qb_username.clone(),
            c.qb_password.clone(),
        )
    })
    .await;

    let refined_host = remove_slash(host);
    let logined = test_login(refined_host.as_str(), username.as_str(), pass.as_str()).await;

    let mut qb = QB.get().unwrap().write().await;
    qb.host = refined_host;
    qb.logined = logined;
}

pub async fn test_login(host: &str, username: &str, pass: &str) -> bool {
    let refined_host = remove_slash(host);
    let mut form = HashMap::new();
    form.insert("username", username);
    form.insert("password", pass);
    let result = http::post(format!("{refined_host}/api/v2/auth/login").as_str())
        .form(&form)
        .disable_cookie()
        .send()
        .await;

    result.is_ok()
}

/// get all torrent infos with CATEGORY
pub async fn get_torrent_info() -> Result<Vec<TorrentInfo>, QbError> {
    let host = get_host().await?;
    let param = HashMap::from([("category", CATEGORY)]);
    http::get(format!("{host}/api/v2/torrents/info"))
        .form(&param)
        .then(async |res| {
            let torrent_info_list: Vec<TorrentInfo> = res.json().await?;
            Ok(torrent_info_list)
        })
        .await
}

/// get the new added torrent hash, and remove the marked tag
pub async fn get_hash() -> Result<String, QbError> {
    let host = get_host().await?;
    let mut param = HashMap::new();
    param.insert("category", CATEGORY);
    param.insert("tag", Tag::NEW.as_ref());

    http::get(format!("{host}/api/v2/torrents/info"))
        .form(&param)
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            // Always occurs when a same torrent is added
            if json_array.is_empty() {
                return Err(QbError {
                    kind: QbErrorKind::NoNewTorrents,
                });
            }
            let hash = json_array[0].get("hash").and_then(|v| v.as_str()).unwrap();
            remove_tag(hash, Tag::NEW).await?;
            Ok(hash.to_string())
        })
        .await
}

async fn manage_tag(hash: &str, tag: Tag, action: &str) -> Result<(), QbError> {
    let host = get_host().await?;
    let param = HashMap::from([("hashes", hash), ("tags", tag.as_ref())]);
    http::post(format!("{host}/api/v2/torrents/{action}"))
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
    let host = get_host().await?;
    http::post(format!("{host}/api/v2/torrents/{action}"))
        .form(&HashMap::from([("hashes", hash)]))
        .then(async |_| Ok(()))
        .await
}

/// start a torrent
pub async fn start(hash: &str) -> Result<(), QbError> {
    manage(hash, "start").await
}

/// stop a torrent
pub async fn stop(hash: &str) -> Result<(), QbError> {
    manage(hash, "stop").await
}

/// delete a torrent
pub async fn delete(hash: &str, delete_files: bool) -> Result<(), QbError> {
    let host = get_host().await?;
    let param = HashMap::from([
        ("hashes", hash),
        ("deleteFiles", if delete_files { "true" } else { "false" }),
    ]);
    http::post(format!("{host}/api/v2/torrents/delete"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// get the state of a torrent
pub async fn get_state(hash: &str) -> Result<String, QbError> {
    let host = get_host().await?;
    http::get(format!("{host}/api/v2/torrents/info"))
        .form(&HashMap::from([("hashes", hash)]))
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            let state = json_array[0].get("state").and_then(|v| v.as_str()).unwrap();
            Ok(state.to_string())
        })
        .await
}

/// get the name of a torrent by its hash
pub async fn get_name(hash: &str) -> Result<String, QbError> {
    let host = get_host().await?;
    http::get(format!("{host}/api/v2/torrents/info"))
        .form(&HashMap::from([("hashes", hash)]))
        .then(async |res| {
            let json_array: Vec<Value> = res.json().await?;
            let name = json_array[0].get("name").and_then(|v| v.as_str()).unwrap();
            Ok(name.to_string())
        })
        .await
}

/// set the download priority of a torrent
/// # Arguments
/// * `hash` - The hash of the torrent.
/// * `priority` - 1 for download, 0 for not download.
/// * `index_list` - the index list of files need to set.
pub async fn set_prio(hash: &str, priority: u8, index_list: &[u32]) -> Result<(), QbError> {
    let host = get_host().await?;
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
    http::post(format!("{host}/api/v2/torrents/filePrio"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// set the task not download, usually used when a new part of task is added
pub async fn set_not_download(task: &TaskValue) -> Result<(), QbError> {
    let (hash, file_num) = (&task.hash, task.file_num);
    set_prio(
        hash.as_str(),
        0,
        (0..file_num).collect::<Vec<u32>>().as_slice(),
    )
    .await
}

/// set the share limit of a torrent
pub async fn set_share_limit(
    hash: &str,
    ratio_limit: f64,
    seeding_time_limit: i32,
) -> Result<(), QbError> {
    let host = get_host().await?;
    let param = HashMap::from([
        ("hashes", hash.to_string()),
        ("ratioLimit", ratio_limit.to_string()),
        ("seedingTimeLimit", seeding_time_limit.to_string()),
        ("inactiveSeedingTimeLimit", "-2".to_string()),
    ]);
    http::post(format!("{host}/api/v2/torrents/setShareLimits"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// export .torrent file to a specified path
pub async fn export(hash: &str, path: &str) -> Result<(), QbError> {
    // wait for the torrent to fetch meta data
    loop {
        let state = get_state(hash).await?;
        if ["stoppedUP", "pausedUP", "stoppedDL", "pausedDL"].contains(&state.as_str()) {
            break;
        }
        sleep(std::time::Duration::from_secs(1)).await;
    }
    let host = get_host().await?;
    http::post(format!("{host}/api/v2/torrents/export"))
        .form(&HashMap::from([("hash", hash)]))
        .then(async |res| {
            let data = res.bytes().await?;
            let mut file = File::create(path)?;
            file.write_all(&data)?;
            Ok(())
        })
        .await
}

/// get the hash list of torrents with a specific tag
pub async fn get_tag_torrent_list(tag: Tag) -> Result<Vec<String>, QbError> {
    let host = get_host().await?;
    let param = HashMap::from([("category", CATEGORY), ("tag", tag.as_ref())]);
    http::get(format!("{host}/api/v2/torrents/info"))
        .form(&param)
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
pub async fn add_by_url(url: &str, save_path: &str) -> Result<(), QbError> {
    let host = get_host().await?;
    let param = HashMap::from([
        ("urls", url),
        ("savepath", save_path),
        ("category", CATEGORY),
        ("tags", Tag::NEW.as_ref()),
        ("stopCondition", "MetadataReceived"),
    ]);
    http::post(format!("{host}/api/v2/torrents/add"))
        .form(&param)
        .then(async |_| Ok(()))
        .await
}

/// add a torrent to qBittorrent by file path,
/// aiming to fast recover from cached .torrrent file
pub async fn add_by_file(
    torrent_path: &str,
    save_path: &str,
    seeding_time_limit: i32,
    ratio_limit: f64,
) -> Result<(), QbError> {
    let host = get_host().await?;
    let file_part = multipart::Part::file(torrent_path).await?;
    let form = multipart::Form::new()
        .part("torrents", file_part)
        .text("category", CATEGORY)
        .text("savepath", save_path.to_string())
        .text("seedingTimeLimit", seeding_time_limit.to_string())
        .text("ratioLimit", ratio_limit.to_string())
        .text("stopped", "true");
    http::post(format!("{host}/api/v2/torrents/add"))
        .multipart(form)
        .then(async |_| Ok(()))
        .await
}

// TODO: test after ui is finished
/// add a torrent to qBittorrent by bytes
pub async fn add_by_bytes(file_name: &str, save_path: &str, data: &[u8]) -> Result<(), QbError> {
    let host = get_host().await?;
    let file_part = multipart::Part::bytes(data.to_vec()).file_name(file_name.to_string());
    let form = multipart::Form::new()
        .part("torrents", file_part)
        .text("savepath", save_path.to_string())
        .text("category", CATEGORY)
        .text("stopped", "true")
        .text("tags", Tag::NEW.as_ref());
    http::post(format!("{host}/api/v2/torrents/add"))
        .multipart(form)
        .then(async |_| Ok(()))
        .await
}
