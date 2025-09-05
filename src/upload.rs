//! deal with upload

use crate::{
    config,
    error::CommonError,
    request::{self, RequestBuilderExt},
    task::{
        Status, TaskValue,
        error::{TaskError, TaskErrorKind},
    },
};
use log::debug;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// upload type, currently support rclone
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "type", content = "job")]
pub enum Uploader {
    /// the containing value is rclone job id
    Rclone(Option<i32>),
}

pub trait UploaderTrait {
    fn upload(task: &mut TaskValue) -> impl Future<Output = Result<(), TaskError>>;
    fn check(task: &mut TaskValue) -> impl Future<Output = Result<bool, TaskError>>;
    fn test(host: &str, username: &str, password: &str) -> impl Future<Output = bool>;
}

impl Uploader {
    /// Check if upload is completed
    pub async fn check(&self, task: &mut TaskValue) -> Result<bool, TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::check(task).await,
        }
    }

    /// Submit upload task
    pub async fn upload(&self, task: &mut TaskValue) -> Result<(), TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::upload(task).await,
        }
    }
}

pub struct Rclone;

impl UploaderTrait for Rclone {
    /// submit upload task to rclone, and store the job ID in the task
    async fn upload(task: &mut TaskValue) -> Result<(), TaskError> {
        let rclone_cfg = config::value().rclone().await;
        let host = &rclone_cfg.rclone_host;
        let username = &rclone_cfg.rclone_username;
        let password = &rclone_cfg.rclone_password;
        let (src, dst) = (
            format!("{}/{}", task.save_path, task.root_dir),
            format!("{}/{}", task.upload_path, task.root_dir),
        );
        let body = json!({
            "srcFs": src,
            "dstFs": dst,
            "_async": true,
            "createEmptySrcDirs": true
        });
        request::post(format!("{host}/sync/copy"))
            .basic_auth(username, Some(password))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .json(&body)
            .then(async |res| {
                let value: Value = res.json().await?;
                if let Some(job_id) = value.get("jobid").and_then(|v| v.as_i64()) {
                    task.uploader = Uploader::Rclone(Some(job_id as i32));
                    Ok(())
                } else {
                    task.status = Status::Error;
                    Err(TaskError {
                        kind: TaskErrorKind::Upload("Rclone job id not found".into()),
                    })
                }
            })
            .await
    }

    async fn check(task: &mut TaskValue) -> Result<bool, TaskError> {
        let rclone_cfg = config::value().rclone().await;
        let host = &rclone_cfg.rclone_host;
        let username = &rclone_cfg.rclone_username;
        let password = &rclone_cfg.rclone_password;
        if let Uploader::Rclone(Some(job_id)) = task.uploader {
            request::post(format!("{host}/job/status"))
                .basic_auth(username, Some(password))
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .json(&json!({
                    "jobid": job_id
                }))
                .then(async |res| {
                    let value: Value = res.json().await?;
                    let success = value.get("success").and_then(|v| v.as_bool()).unwrap();
                    let finished = value.get("finished").and_then(|v| v.as_bool()).unwrap();

                    // finished but not successful means there were errors
                    if finished && !success {
                        task.status = Status::Error;
                        return Err(TaskError {
                            kind: TaskErrorKind::Upload("Rclone job finished with errors".into()),
                        });
                    }
                    Ok(success)
                })
                .await
        } else {
            task.status = Status::Error;
            Err(TaskError {
                kind: TaskErrorKind::Upload("No rclone job ID found".into()),
            })
        }
    }

    async fn test(host: &str, username: &str, password: &str) -> bool {
        let res = request::post(format!("{host}/core/version"))
            .basic_auth(username, Some(password))
            .then::<bool, CommonError, _, _>(async |res| {
                let value: Value = res.json().await?;
                let version = value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                debug!("Rclone version: {}", version);
                Ok(true)
            })
            .await;
        res.is_ok()
    }
}
