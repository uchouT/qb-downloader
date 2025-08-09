use std::sync::{OnceLock, Arc, atomic::AtomicBool};

use crate::Error;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
pub struct Application {
    pub running: Arc<AtomicBool>,
}

impl Application {
    pub fn new() -> Self {
        Application { 
            running: Arc::new(AtomicBool::new(true))
        }
    }
    fn init(&mut self) -> Result<(), Error> {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
        });
        Ok(())
    }
}
