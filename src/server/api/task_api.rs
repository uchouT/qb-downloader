//! end point at "/api/task"
//!
//! GET: get task status
//! POST: add new task
//! PUT: manage tasks - pause, start/resume
//! DELETE: delete task
use crate::{
    config::{self, strip_slash},
    error::{TaskError, format_error_chain},
    qb,
    server::{
        ResultResponse,
        api::{from_json_owned, get_option_param, get_param_map, get_required_param},
        error::ServerError,
    },
    task::{self, task_map},
    upload::Uploader,
};
use anyhow::Context;
use hyper::{Method, Response, StatusCode};
use log::{error, warn};
use serde::Deserialize;

use super::{Action, BoxBody, Req, ServerResult, torrent_api::TorrentRes};

#[derive(Debug, Default)]
pub struct TaskAPI;

impl Action for TaskAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        if !qb::is_logined() {
            return Ok(ResultResponse::success());
        }

        match *req.method() {
            Method::GET => get(),
            Method::POST => post(req).await,
            Method::PUT => put(req).await,
            Method::DELETE => delete(req).await,
            _ => Ok(ResultResponse::error_with_code(
                StatusCode::METHOD_NOT_ALLOWED,
            )),
        }
    }
}

fn get() -> ServerResult<Response<BoxBody>> {
    let data = task::task_map();
    if data.is_empty() {
        Ok(ResultResponse::success())
    } else {
        Ok(ResultResponse::success_data(data.clone()))
    }
}

/// Add a new task
/// [`TaskReq`] is deserialized from the request body
/// # Precondition
/// - save_path is valid when passing [`TorrentRes`], meaning the path exists
async fn post(req: Req) -> ServerResult<Response<BoxBody>> {
    let mut task_req: TaskReq = from_json_owned(req).await?;
    if !task_req.custom_content {
        task_req.selected_file_index = None;
    } else if let Some(ref selected_file_index) = task_req.selected_file_index
        && selected_file_index.is_empty()
    {
        return Ok(ResultResponse::error_msg("Selected none content"));
    }

    let c = config::value();
    if task_req.upload_path.is_empty() {
        let general_cfg = &c.general;
        if general_cfg.default_upload_path.is_empty() {
            return Err(ServerError::MissingParams("upload_path"));
        } else {
            task_req.upload_path = general_cfg.default_upload_path.clone();
        }
    }
    let seeding_time_limit = task_req
        .seeding_time_limit
        .or(c.qb.default_seeding_time_limit)
        .ok_or(ServerError::MissingParams("seeding_time_limit"))?;

    let ratio_limit = task_req
        .ratio_limit
        .or(c.qb.default_ratio_limit)
        .ok_or(ServerError::MissingParams("ratio_limit"))?;

    if let Err(e) = task::add(
        task_req.torrent_res.hash,
        task_req.torrent_res.torrent_name,
        task_req.torrent_res.save_path,
        task_req.upload_path,
        task_req.upload_type,
        task_req.selected_file_index,
        task_req.max_size * 1024 * 1024 * 1024, // default in GB
        ratio_limit,
        seeding_time_limit,
    )
    .await
    {
        if let TaskError::OverSize = e {
            let msg = "Selected files exceed maximum length";
            warn!("{msg}");
            return Ok(ResultResponse::error_msg(msg));
        }
        let msg = "Failed to add a task";
        error!("{msg}\n{}", format_error_chain(e));
        return Ok(ResultResponse::error_msg(msg));
    }
    Ok(ResultResponse::success_msg("Task added successfully"))
}

async fn put(req: Req) -> ServerResult<Response<BoxBody>> {
    let (hash, manipulate_type, forced) = {
        let params = get_param_map(&req).ok_or(ServerError::MissingParams("hash or type"))?;
        (
            get_required_param::<String>(&params, "hash")?,
            get_required_param::<String>(&params, "type")?,
            get_option_param::<bool>(&params, "forced"),
        )
    };
    match manipulate_type.as_str() {
        "start" => start_task(&hash, forced).await?,
        "stop" => task::stop(&hash).await.context("Failed to stop task")?,
        _ => {
            return Ok(ResultResponse::bad_request(Some("Invalid type".into())));
        }
    }
    Ok(ResultResponse::success())
}

async fn delete(req: Req) -> ServerResult<Response<BoxBody>> {
    let hash = {
        let params = get_param_map(&req).ok_or(ServerError::MissingParams("hash"))?;
        get_required_param::<String>(&params, "hash")?
    };
    if let Err(e) = task::delete(&hash, true).await {
        let msg = "Failed to delete task";
        error!("{msg}\n{}", format_error_chain(e));
        return Ok(ResultResponse::error_msg(msg));
    }
    Ok(ResultResponse::success())
}

#[derive(Debug, Deserialize)]
pub struct TaskReq {
    pub torrent_res: TorrentRes,
    pub upload_type: Uploader,
    #[serde(deserialize_with = "strip_slash")]
    pub upload_path: String,
    pub max_size: i64,
    pub seeding_time_limit: Option<i32>,
    pub ratio_limit: Option<f64>,
    pub custom_content: bool,
    pub selected_file_index: Option<Vec<usize>>,
}

async fn start_task(hash: &str, forced: Option<bool>) -> ServerResult<()> {
    let forced = forced.unwrap_or(false);
    let task = task_map()
        .get(hash)
        .cloned()
        .ok_or(anyhow::anyhow!("Task not found"))?;
    let status = task.state().status;

    match status {
        // start from paused state
        task::Status::Paused => task::start(task).await.context("Failed to start task")?,
        // start from error state
        task::Status::Error => {
            let kind = task.error_info().as_ref().as_ref().unwrap().kind;
            if forced && !kind.skipable() {
                Err(anyhow::anyhow!(
                    "This error is not skipable, cannot force start"
                ))?;
            }

            task::resume(task, kind, forced)
                .await
                .context("Failed to resume task")?;
        }
        _ => Err(anyhow::anyhow!("Task is not in a paused or error state"))?,
    }
    Ok(())
}
