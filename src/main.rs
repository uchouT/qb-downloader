use qb_downloader::app::{self, AppError};
fn main() -> Result<(), AppError> {
    let (addr, port) = app::init()?;
    app::run(addr, port)?;
    Ok(())
}
