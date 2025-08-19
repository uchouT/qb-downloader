use std::convert::Infallible;

use futures::try_join;
use log::{error, info};
use tokio::sync::broadcast;

use crate::{Entity, Error, config, qb, server, task};

pub struct Application;

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        Self
    }

    fn create_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    }

    pub fn run(&self, port: u16) -> Result<(), Error> {
        let runtime = Self::create_runtime();
        runtime.block_on(async move {
            let (shutdown_tx, _) = broadcast::channel::<()>(1);

            tokio::spawn(Self::wait_for_signal(shutdown_tx.clone()));
            let result = try_join!(
                task::handle::run(shutdown_tx.subscribe()),
                server::run(shutdown_tx.subscribe(), port)
            );

            if let Err(e) = shutdown().await {
                error!("Error occurred while shutting down application: {e}");
            }
            result
        })?;
        Ok(())
    }

    async fn wait_for_signal(shutdown_tx: broadcast::Sender<()>) {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        let _ = shutdown_tx.send(());
    }
}

/// Task before shutdown the application
async fn shutdown() -> Result<(), Infallible> {
    if qb::is_logined().await {
        info!("Removing waited torrents");
        let _ = task::clean_waited().await;
    }
    if let Err(e) = config::Config::save().await {
        error!("Failed to save configuration: {e}");
    }
    info!("Application shutdown completed");
    Ok(())
}
