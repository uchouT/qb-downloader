use qb_downloader_rust::{Entity, app, config::Config, error::Error, task::Task};
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let port = init()?;
    let application = app::Application::new();
    application.run(port)?;
    Ok(())
}

fn init() -> Result<u16, Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut config_path = None;
    let mut task_path = None;
    let mut port: u16 = 7845;
    args.iter()
        .enumerate()
        .for_each(|(i, arg)| match arg.as_str() {
            "--config" | "-c" => {
                config_path = Some(PathBuf::from(args.get(i + 1).expect("invalid arguments")));
            }
            "--task-path" => {
                task_path = Some(PathBuf::from(args.get(i + 1).expect("invalid arguments")));
            }
            "--log-level" => unsafe {
                std::env::set_var("RUST_LOG", args.get(i + 1).expect("invalid arguments"));
            },
            "--port" | "-p" => {
                port = args
                    .get(i + 1)
                    .expect("invalid arguments")
                    .parse()
                    .expect("invalid port");
            }
            _ => {}
        });
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
    Config::init(config_path)?;
    Task::init(task_path)?;
    Ok(port)
}
