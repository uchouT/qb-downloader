use qb_downloader::app;
use anyhow::Error;
fn main() -> Result<(), Error> {
    let port = app::init()?;
    app::run(port)?;
    Ok(())
}
