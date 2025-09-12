use futures_util::{FutureExt, select, try_join};
use log::{error, info};
use std::{convert::Infallible, path::PathBuf};
use tokio::{
    signal::{self, unix::SignalKind},
    sync::broadcast,
};

use crate::{Error, VERSION, config, qb, server, task};
const PORT: u16 = 7845;

pub fn run(port: u16) -> Result<(), Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    runtime.block_on(async move {
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        tokio::spawn(wait_for_signal(shutdown_tx.clone()));

        let task_service = tokio::spawn(task::handle::run(shutdown_tx.subscribe()));
        let server_service = tokio::spawn(server::run(shutdown_tx.subscribe(), port));

        let result = match try_join!(task_service, server_service) {
            Ok((res1, res2)) => {
                res1?;
                res2?;
                Ok::<(), Error>(())
            }
            Err(e) => {
                error!("A service encountered an error: {e}");
                Ok(())
            }
        };
        let _ = cleanup().await;
        result
    })?;
    Ok(())
}

async fn wait_for_signal(shutdown_tx: broadcast::Sender<()>) {
    let mut sigterm =
        signal::unix::signal(SignalKind::terminate()).expect("Failed to set up signal handler");

    select! {
        _ = signal::ctrl_c().fuse() =>  {},
        _ =  sigterm.recv().fuse()=> {},
    }
    let _ = shutdown_tx.send(());
}

/// Task before shutdown the application
async fn cleanup() -> Result<(), Infallible> {
    if qb::is_logined() {
        info!("Removing waited torrents");
        let _ = task::clean_waited().await;
    }
    if let Err(e) = config::save().await {
        error!("Failed to save configuration: {e}");
    }
    info!("Application shutdown completed");
    Ok(())
}

pub fn init() -> Result<u16, Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut config_path = None;
    let mut task_path = None;
    let mut port = PORT;
    args.iter()
        .enumerate()
        .for_each(|(i, arg)| match arg.as_str() {
            "--config" | "-c" => {
                config_path = Some(PathBuf::from(args.get(i + 1).expect("invalid arguments")));
            }
            "--task-path" => {
                task_path = Some(PathBuf::from(args.get(i + 1).expect("invalid arguments")));
            }
            "--log-level" => unsafe {
                std::env::set_var("RUST_LOG", args.get(i + 1).expect("invalid arguments"));
            },
            "--port" | "-p" => {
                port = args
                    .get(i + 1)
                    .expect("invalid arguments")
                    .parse()
                    .expect("invalid port");
            }
            _ => {}
        });
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    pretty_env_logger::init();
    info!("qb-downloader v{VERSION} starting...");
    config::init(config_path)?;
    task::init(task_path)?;
    Ok(port)
}
