//! deal with upload

use reqwest::header::{CONTENT_TYPE, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    Entity, Error, InstanceEntity,
    config::Config,
    http::{self, Extra},
    task::Task,
};

/// upload type, currently support rclone
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Uploader {
    /// the containing value is rclone job id
    Rclone(Option<i32>),
}

pub trait UploadCheck {
    /// submit async upload task
    async fn upload(&self, task: &mut Task) -> Result<(), Error>;
    /// check if the upload task is finished
    async fn check(&self, task: &mut Task) -> Result<bool, Error>;
}

impl UploadCheck for Uploader {
    async fn check(&self, task: &mut Task) -> Result<bool, Error> {
        match self {
            Uploader::Rclone(_) => Rclone::check(task).await,
        }
    }

    async fn upload(&self, task: &mut Task) -> Result<(), Error> {
        match self {
            Uploader::Rclone(_) => Rclone::upload(task).await,
        }
    }
}

struct Rclone;

impl Rclone {
    /// submit upload task to rclone, and store the job ID in the task
    async fn upload(task: &mut Task) -> Result<(), Error> {
        let (host, username, password) = Config::read(|config| {
            (
                config.rclone_host.clone(),
                config.rclone_username.clone(),
                config.rclone_password.clone(),
            )
        });
        let (src, dst) = task.read(|value| {
            (
                format!("{}/{}", value.save_path, value.root_dir),
                format!("{}/{}", value.upload_path, value.root_dir),
            )
        });
        let body = json!({
            "srcFs": src,
            "dstFs": dst,
            "_async": true,
            "createEmptySrcDirs": true
        });
        http::post(format!("{}/sync/copy", host))
            .basic_auth(username, Some(password))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .json(&body)
            .then(async |res| {
                let value: Value = res.json().await?;
                if let Some(job_id) = value.get("jobid").and_then(|v| v.as_i64()) {
                    task.write(|value| {
                        value.uploader = Uploader::Rclone(Some(job_id as i32));
                    });
                    Ok(())
                } else {
                    Err(Error::Upload(
                        "Failed to get job ID from rclone response".into(),
                    ))
                }
            })
            .await
    }

    async fn check(task: &mut Task) -> Result<bool, Error> {
        let (host, username, password) = Config::read(|config| {
            (
                config.rclone_host.clone(),
                config.rclone_username.clone(),
                config.rclone_password.clone(),
            )
        });
        if let Uploader::Rclone(Some(job_id)) = task.read(|v| v.uploader.clone()) {
            http::post(format!("{}/job/status", host))
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
                        return Err(Error::Upload("Rclone job finished with errors".into()));
                    }
                    Ok(success)
                })
                .await
        } else {
            Err(Error::Upload("No rclone job ID found".into()))
        }
    }
}
