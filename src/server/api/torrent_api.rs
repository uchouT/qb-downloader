//! torrent api
//! # POST:
//! add torrent, if content-type is multipart/form-data, add by file,
//! if application/json, add by url, request body is [`TorrentReq`]
//! GET: get torrent content tree
//! DELETE: delete torrent. which is not added as task

use std::borrow::Cow;

use crate::{
    bencode::{self, BencodeError, FileNode},
    config::{self, strip_slash},
    errors::{IntoContextedError, TargetContextedResult},
    qb, remove_slash,
    server::{
        ResultResponse,
        api::{
            ReqExt, from_json, get_json_body, get_option_param, get_param_map, get_required_param,
        },
        error::ServerError,
    },
    task::{self, error::TaskError, get_torrent_path},
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
            Method::PUT => put(req).await,
            _ => Ok(ResultResponse::error_with_code(
                StatusCode::METHOD_NOT_ALLOWED,
            )),
        }
    }
}

async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let is_file = req.is_multipart();
    let (file, url, save_path) = if is_file {
        get_add_by_file_param(req).await?
    } else {
        get_add_by_url_param(req).await?
    };

    let hash = match task::add_torrent(
        // TODO: zero-copy here
        file.map(|f| {
            let bytes: Vec<_> = f.into();
            Cow::Owned(bytes)
        }),
        &url,
        &save_path,
    )
    .await
    {
        Ok(h) => h,
        Err(TaskError::Abort) => return Ok(ResultResponse::success()),
        Err(e) => Err(e).convert_then_add_context("Failed to add torrent")?,
    };

    if is_file {
        // response with full torrent info
        let torrent_name = get_torrent_name_from_hash(&hash).await?;
        let res = TorrentRes {
            torrent_name,
            hash,
            save_path,
        };
        Ok(ResultResponse::success_data(res))
    } else {
        // response with hash and save_path first, fetching metadata asynchronously
        let res = AsyncTorrentRes { hash, save_path };
        Ok(ResultResponse::success_data(res))
    }
}

async fn get_torrent_name_from_hash(hash: &str) -> ServerResult<String> {
    let torrent_name = bencode::get_torrent_name(&hash).await.map_err(|e| {
        // clean added torrent
        tokio::spawn(task::delete(hash.to_string(), false));

        if let BencodeError::SingleFile = e {
            ServerError::create_internal("Not a multi-file torrent")
        } else {
            ServerError::from(e.into_contexted_error("Failed to parse torrent"))
        }
    })?;
    Ok(torrent_name)
}

async fn get_add_by_file_param(req: Req) -> ServerResult<(Option<Bytes>, String, String)> {
    let mut multipart = req.into_multipart()?;
    let mut data = None;
    let mut save_path = None;
    let mut file_name = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .convert_then_add_context("Failed to read multipart field")?
    {
        match field.name() {
            Some("torrent") => {
                file_name = field.file_name().map(|s| s.to_string());
                data = Some(
                    field
                        .bytes()
                        .await
                        .convert_then_add_context("Failed to read torrent file")?,
                );
            }
            Some("save_path") => {
                let path = field
                    .text()
                    .await
                    .convert_then_add_context("Failed to read save_path")?;
                if !remove_slash(&path).is_empty() {
                    save_path = Some(path);
                }
            }
            _ => {}
        }
    }

    if data.is_none() {
        return Err(ServerError::MissingParams("torrent file"));
    }
    if save_path.is_none() {
        let default_path = config::value().qb.default_save_path.clone();
        if default_path.is_empty() {
            return Err(ServerError::MissingParams("save_path"));
        } else {
            save_path = Some(default_path);
        }
    }
    Ok((data, file_name.unwrap_or_default(), save_path.unwrap()))
}

async fn get_add_by_url_param(req: Req) -> ServerResult<(Option<Bytes>, String, String)> {
    let body = get_json_body(req).await?;
    let torrent_req: TorrentReq = from_json(&body)?;
    let save_path = {
        if torrent_req.save_path.is_empty() {
            let default_path = config::value().qb.default_save_path.clone();
            if default_path.is_empty() {
                return Err(ServerError::MissingParams("save_path"));
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
        let params = get_param_map(&req).ok_or(ServerError::MissingParams("hash"))?;
        get_required_param::<String>(&params, "hash")?
    };
    let torrent_path = get_torrent_path(&hash);
    let file_tree = FileNode::get_tree(&torrent_path)
        .await
        .convert_then_add_context("Failed to get torrent content tree")?;
    Ok(ResultResponse::success_data(vec![file_tree]))
}

/// delete aborted torrent (not added as task)
async fn delete(req: Req) -> ServerResult<Response<BoxBody>> {
    let (hash, fetching) = {
        let params = get_param_map(&req).ok_or(ServerError::MissingParams("hash"))?;
        (
            get_required_param::<String>(&params, "hash")?,
            get_option_param::<bool>(&params, "fetching"),
        )
    };

    if matches!(fetching, Some(true)) {
        task::cancel_fetching(&hash)
            .await
            .convert_then_add_context("Failed to cancel torrent adding")?;
    } else {
        task::delete(&hash, false)
            .await
            .convert_then_add_context("Failed to delete torrent")?;
    }
    Ok(ResultResponse::success())
}

async fn put(req: Req) -> ServerResult<Response<BoxBody>> {
    let hash = {
        let params = get_param_map(&req).ok_or(ServerError::MissingParams("hash"))?;
        get_required_param::<String>(&params, "hash")?
    };
    task::block_fetching(&hash)
        .await
        .convert_then_add_context("Failed to waiting fetching metadata")?;
    let torrent_name = get_torrent_name_from_hash(&hash).await?;
    Ok(ResultResponse::success_data(torrent_name))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TorrentRes {
    pub torrent_name: String,
    pub hash: String,
    /// save path should be nomalized before serialized
    pub save_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AsyncTorrentRes {
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
