use anyhow::Error;
use qb_downloader::app;
fn main() -> Result<(), Error> {
    let port = app::init()?;
    app::run(port)?;
    Ok(())
}
