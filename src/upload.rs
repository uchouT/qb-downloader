//! deal with upload

use std::sync::{Arc, RwLock};

use crate::{
    config,
    request::{self, RequestError},
    task::{Status, TaskValue, error::TaskError},
};
use log::debug;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// upload type, currently support rclone
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "job")]
pub enum Uploader {
    /// the containing value is rclone job id
    Rclone(RwLock<Option<i32>>),
}

pub trait UploaderTrait {
    fn upload(task: Arc<TaskValue>) -> impl Future<Output = Result<(), TaskError>>;
    fn check(task: Arc<TaskValue>) -> impl Future<Output = Result<bool, TaskError>>;
    fn test(host: &str, username: &str, password: &str) -> impl Future<Output = bool>;
}

impl Uploader {
    /// Check if upload is completed
    pub async fn check(&self, task: Arc<TaskValue>) -> Result<bool, TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::check(task).await,
        }
    }

    /// Submit upload task
    pub async fn upload(&self, task: Arc<TaskValue>) -> Result<(), TaskError> {
        match self {
            Uploader::Rclone(_) => Rclone::upload(task).await,
        }
    }
}

pub struct Rclone;

impl UploaderTrait for Rclone {
    /// submit upload task to rclone, and store the job ID in the task
    async fn upload(task: Arc<TaskValue>) -> Result<(), TaskError> {
        let rclone_cfg = &config::value().rclone;
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
            .basic_auth(username, password)
            .json(body)
            .send_and_then(async |res| {
                let value: Value = res.json().await.map_err(RequestError::from)?;
                if let Some(job_id) = value.get("jobid").and_then(|v| v.as_i64()) {
                    let Uploader::Rclone(id) = &task.uploader;
                    *id.write().unwrap() = Some(job_id as i32);
                    Ok(())
                } else {
                    // TODO: more specific error
                    task.state_mut().status = Status::Error;
                    Err(TaskError::Upload("Rclone job id not found"))
                }
            })
            .await
    }

    async fn check(task: Arc<TaskValue>) -> Result<bool, TaskError> {
        let rclone_cfg = &config::value().rclone;
        let host = &rclone_cfg.rclone_host;
        let username = &rclone_cfg.rclone_username;
        let password = &rclone_cfg.rclone_password;

        let job_id = {
            let Uploader::Rclone(job_id_opt) = &task.uploader;
            if job_id_opt.read().unwrap().is_none() {
                task.state_mut().status = Status::Error;
                return Err(
                    // TODO: make it more specific
                    TaskError::Upload("No rclone job ID found"),
                );
            }
            job_id_opt.read().unwrap().unwrap()
        };

        request::post(format!("{host}/job/status"))
            .basic_auth(username, password)
            .json(json!({
                "jobid": job_id
            }))
            .send_and_then(async |res| {
                let value: Value = res.json().await.map_err(RequestError::from)?;
                let success = value.get("success").and_then(|v| v.as_bool()).unwrap();
                let finished = value.get("finished").and_then(|v| v.as_bool()).unwrap();

                // finished but not successful means there were errors
                if finished && !success {
                    task.state_mut().status = Status::Error;
                    return Err(
                        // TODO: make it more specific
                        TaskError::Upload("Rclone job finished with errors"),
                    );
                }
                Ok(success)
            })
            .await
    }

    async fn test(host: &str, username: &str, password: &str) -> bool {
        let res = request::post(format!("{host}/core/version"))
            .basic_auth(username, password)
            .send_and_then::<_, RequestError, _, _>(async |res| {
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
