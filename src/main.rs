use qb_downloader::{app, error::Error};

fn main() -> Result<(), Error> {
    let port = app::init()?;
    app::Application::run(port)?;
    Ok(())
}
