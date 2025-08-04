use std::error::Error;
use qb_downloader_rust::config;
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    config::init(None);
    Ok(())
}
