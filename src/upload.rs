//! deal with upload

use reqwest::header::{CONTENT_TYPE, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    Entity,
    config::Config,
    http::{self, Extra},
    task::{Status, TaskValue, error::*},
};

/// upload type, currently support rclone
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum Uploader {
    /// the containing value is rclone job id
    Rclone(Option<i32>),
}

pub trait UploadCheck {
    /// submit async upload task
    fn upload(&self, task: &mut TaskValue) -> impl Future<Output = Result<(), TaskError>>;
    /// check if the upload task is finished
    fn check(&self, task: &mut TaskValue) -> impl Future<Output = Result<bool, TaskError>>;
}

impl UploadCheck for Uploader {
    async fn check(&self, task: &mut TaskValue) -> Result<bool, TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::check(task).await,
        }
    }

    async fn upload(&self, task: &mut TaskValue) -> Result<(), TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::upload(task).await,
        }
    }
}

struct Rclone;

impl Rclone {
    /// submit upload task to rclone, and store the job ID in the task
    async fn upload(task: &mut TaskValue) -> Result<(), TaskError> {
        let (host, username, password) = Config::read(|config| {
            (
                config.rclone_host.clone(),
                config.rclone_username.clone(),
                config.rclone_password.clone(),
            )
        })
        .await;
        let (src, dst) = (
            format!("{}:{}", task.upload_path, task.name),
            format!("{}:{}", task.save_path, task.name),
        );
        let body = json!({
            "srcFs": src,
            "dstFs": dst,
            "_async": true,
            "createEmptySrcDirs": true
        });
        http::post(format!("{host}/sync/copy"))
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
        let (host, username, password) = Config::read(|config| {
            (
                config.rclone_host.clone(),
                config.rclone_username.clone(),
                config.rclone_password.clone(),
            )
        })
        .await;
        if let Uploader::Rclone(Some(job_id)) = task.uploader {
            http::post(format!("{host}/job/status"))
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
}
