use qb_downloader_rust::config;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    
    env_logger::init();
    config::init(None)?;
    
    println!("Application started successfully!");
    
    Ok(())
}
