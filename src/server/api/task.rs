//! end point at "/api/task"
//!
//! GET: get task status
//! POST: manage tasks - pause, start/resume
//! PUT: add new task
//! DELETE: delete task
use crate::{
    Entity,
    config::{Config, strip_slash},
    qb,
    server::{
        ResultResponse,
        api::{from_json_owned, get_param_map, get_required_param},
        error::{ServerError, ServerErrorKind},
    },
    task,
    upload::Uploader,
};
use hyper::{Method, Response, StatusCode};
use log::error;
use serde::Deserialize;

use super::{Action, BoxBody, Req, ServerResult, torrent::TorrentRes};

#[derive(Debug, Default)]
pub struct TaskAPI;

impl Action for TaskAPI {
    async fn execute(&self, req: Req) -> ServerResult<Response<BoxBody>> {
        if !qb::is_logined().await {
            return Ok(ResultResponse::success());
        }

        match *req.method() {
            Method::GET => get().await,
            Method::POST => post(req).await,
            Method::PUT => put(req).await,
            Method::DELETE => delete(req).await,
            _ => Ok(ResultResponse::error_with_code(
                StatusCode::METHOD_NOT_ALLOWED,
            )),
        }
    }
}

async fn get() -> ServerResult<Response<BoxBody>> {
    let data = task::Task::get_task_map().await;
    if data.is_empty() {
        Ok(ResultResponse::success())
    } else {
        Ok(ResultResponse::success_data(data))
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
    }
    Config::read(|c| {
        if task_req.upload_path.is_empty() {
            if c.default_upload_path.is_empty() {
                return Err(ServerError {
                    kind: ServerErrorKind::MissingParams("upload_path"),
                });
            } else {
                task_req.upload_path = c.default_upload_path.clone();
            }
        }
        task_req
            .seeding_time_limit
            .or(c.default_seeding_time_limit)
            .ok_or(ServerError {
                kind: ServerErrorKind::MissingParams("seeding_time_limit"),
            })?;
        task_req
            .ratio_limit
            .or(c.default_ratio_limit)
            .ok_or(ServerError {
                kind: ServerErrorKind::MissingParams("ratio_limit"),
            })?;
        Ok::<(), ServerError>(())
    })
    .await?;
    if let Err(e) = task::add(
        task_req.torrent_res.hash,
        task_req.torrent_res.torrent_name,
        task_req.torrent_res.save_path,
        task_req.upload_path,
        task_req.upload_type,
        task_req.selected_file_index,
        task_req.max_size * 1024 * 1024 * 1024, // default in GB
        task_req.ratio_limit.unwrap(),
        task_req.seeding_time_limit.unwrap(),
    )
    .await
    {
        error!("{e}");
        return Ok(ResultResponse::error_msg("Failed to add task"));
    }
    Ok(ResultResponse::success_msg("Task added successfully"))
}

async fn put(req: Req) -> ServerResult<Response<BoxBody>> {
    let (hash, manipulate_type) = {
        let params = get_param_map(&req).ok_or(ServerError {
            kind: ServerErrorKind::MissingParams("hash or type"),
        })?;
        (
            get_required_param::<String>(&params, "hash")?,
            get_required_param::<String>(&params, "type")?,
        )
    };
    match manipulate_type.as_str() {
        "start" => task::start(&hash).await?,
        "stop" => task::stop(&hash).await?,
        _ => {
            return Ok(ResultResponse::bad_request(Some("Invalid type")));
        }
    }
    Ok(ResultResponse::success())
}
async fn delete(req: Req) -> ServerResult<Response<BoxBody>> {
    let hash = {
        let params = get_param_map(&req).ok_or(ServerError {
            kind: ServerErrorKind::MissingParams("hash"),
        })?;
        get_required_param::<String>(&params, "hash")?
    };
    task::delete(&hash, true).await?;
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
