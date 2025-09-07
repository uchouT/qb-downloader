//! torrent api
//! # POST:
//! add torrent, if content-type is multipart/form-data, add by file,
//! if application/json, add by url, request body is [`TorrentReq`]
//! GET: get torrent content tree
//! DELETE: delete torrent. which is not added as task

use crate::{
    bencode::{self, FileNode},
    config::{self, strip_slash},
    qb::{self, Tag},
    remove_slash,
    server::{
        ResultResponse,
        api::{
            ReqExt, from_json, get_json_body, get_option_param, get_param_map, get_required_param,
        },
        error::{ServerError, ServerErrorKind},
    },
    task::{self, error::TaskErrorKind, get_torrent_path},
};

use hyper::{Method, Response, StatusCode, body::Bytes};
use serde::{Deserialize, Serialize};

use super::{Action, BoxBody, Req, ServerResult};

#[derive(Debug, Default)]
pub struct TorrentAPI;

impl Action for TorrentAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        if !qb::is_logined() {
            return Ok(ResultResponse::error_msg("Qbittorrent is not logged in"));
        }
        match *req.method() {
            Method::POST => post(req).await,
            Method::GET => get(req).await,
            Method::DELETE => delete(req).await,
            _ => Ok(ResultResponse::error_with_code(
                StatusCode::METHOD_NOT_ALLOWED,
            )),
        }
    }
}

async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let (file, url, save_path) = if req.is_multipart() {
        add_by_file(req).await?
    } else {
        add_by_url(req).await?
    };
    let hash = match task::add_torrent(file.as_deref(), &url, &save_path).await {
        Ok(h) => h,
        Err(e) => {
            if let TaskErrorKind::Abort = e.kind {
                return Ok(ResultResponse::success());
            } else {
                return Err(ServerError::from(e));
            }
        }
    };
    let torrent_name = bencode::get_torrent_name(&hash).await.inspect_err(|_| {
        let hash = hash.clone();
        tokio::spawn(async move {
            let _ = task::delete(&hash, false).await;
        });
    })?;
    let res = TorrentRes {
        torrent_name,
        hash,
        save_path,
    };
    Ok(ResultResponse::success_data(res))
}

async fn add_by_file(req: Req) -> ServerResult<(Option<Bytes>, String, String)> {
    let mut multipart = req.into_multipart()?;
    let mut data = None;
    let mut save_path = None;
    let mut file_name = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("torrent") => {
                file_name = field.file_name().map(|s| s.to_string());
                data = Some(field.bytes().await?);
            }
            Some("save_path") => {
                let path = field.text().await?;
                if !remove_slash(&path).is_empty() {
                    save_path = Some(path);
                }
            }
            _ => {}
        }
    }

    if data.is_none() {
        return Err(ServerError {
            kind: ServerErrorKind::MissingParams("torrent file"),
        });
    }
    if save_path.is_none() {
        let default_path = config::value().qb.default_save_path.clone();
        if default_path.is_empty() {
            return Err(ServerError {
                kind: ServerErrorKind::MissingParams("save_path"),
            });
        } else {
            save_path = Some(default_path);
        }
    }
    Ok((data, file_name.unwrap_or_default(), save_path.unwrap()))
}

async fn add_by_url(req: Req) -> ServerResult<(Option<Bytes>, String, String)> {
    let body = get_json_body(req).await?;
    let torrent_req: TorrentReq = from_json(&body)?;
    let save_path = {
        if torrent_req.save_path.is_empty() {
            let default_path = config::value().qb.default_save_path.clone();
            if default_path.is_empty() {
                return Err(ServerError {
                    kind: ServerErrorKind::MissingParams("save_path"),
                });
            } else {
                default_path
            }
        } else {
            torrent_req.save_path
        }
    };
    let url = torrent_req.url.trim();
    Ok((None, url.to_string(), save_path))
}

/// get torrent content tree
async fn get(req: Req) -> ServerResult<Response<BoxBody>> {
    let hash = {
        let params = get_param_map(&req).ok_or(ServerError {
            kind: ServerErrorKind::MissingParams("hash"),
        })?;
        get_required_param::<String>(&params, "hash")?
    };
    let torrent_path = get_torrent_path(&hash);
    let file_tree = FileNode::get_tree(&torrent_path).await?;
    Ok(ResultResponse::success_data(vec![file_tree]))
}

/// delete aborted torrent (not added as task)
async fn delete(req: Req) -> ServerResult<Response<BoxBody>> {
    let hash = {
        if let Some(params) = get_param_map(&req) {
            get_option_param::<String>(&params, "hash")
        } else {
            None
        }
    };
    if let Some(hash) = hash {
        task::delete(&hash, false).await?;
    } else {
        let hash_list = qb::get_tag_torrent_list(Tag::New).await?;
        let hash = hash_list.join("|");
        qb::delete(hash.as_str(), true).await?;
    }
    Ok(ResultResponse::success())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentRes {
    pub torrent_name: String,
    pub hash: String,
    /// save path should be nomalized before serialized
    pub save_path: String,
}

/// add torrent by url
#[derive(Debug, Deserialize)]
pub struct TorrentReq<'a> {
    #[serde(borrow)]
    pub url: &'a str,
    #[serde(deserialize_with = "strip_slash")]
    pub save_path: String,
}
