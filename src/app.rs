use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use futures::try_join;

use crate::{Error, task};

pub struct Application {
    pub running: Arc<AtomicBool>,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Application {
    fn create_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    }

    pub fn run(&self) -> Result<(), Error> {
        let runtime = Self::create_runtime();
        self.running.store(true, Ordering::Relaxed);
        runtime.block_on(async move { try_join!(task::handle::run(self.running.clone())) })?;
        Ok(())
    }
}
