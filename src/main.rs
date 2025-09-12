use qb_downloader::{app, error::Error};

fn main() -> Result<(), Error> {
    let port = app::init()?;
    app::run(port)?;
    Ok(())
}
