use qb_downloader_rust::{app, error::Error};

fn main() -> Result<(), Error> {
    let port = app::init()?;
    let application = app::Application::new();
    application.run(port)?;
    Ok(())
}
