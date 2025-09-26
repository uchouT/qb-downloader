use anyhow::Error;
use qb_downloader::app;
fn main() -> Result<(), Error> {
    let (addr, port) = app::init()?;
    app::run(addr, port)?;
    Ok(())
}
