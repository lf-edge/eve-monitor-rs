mod actions;
mod application;
mod events;
mod ipc;
mod mainwnd;
mod terminal;
mod traits;
mod ui;

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;
use application::Application;
use log::{info, LevelFilter};

fn init_logging() -> log2::Handle {
    let log_dir = if let Ok(_dir) = std::env::var("XDG_RUNTIME_DIR") {
        // store log in the current directory for convenience
        // we use XDG_RUNTIME_DIR to detect the fact that we are running on desktop linux
        PathBuf::from("./")
    } else {
        PathBuf::from("/run")
    };

    let log_file = log_dir.join("./monitor.log").to_string_lossy().to_string();

    let handle = log2::open(&log_file)
        .size(10 * 1024 * 1024)
        .rotate(20)
        .tee(false) // no console output
        .module(true)
        .level(LevelFilter::Debug)
        .start();

    info!("Logging initialized: {:?}", log_file);

    handle
}
#[tokio::main]
async fn main() -> Result<()> {
    let _log2 = init_logging();

    let mut app = Application::new()?;
    app.run().await?;
    Ok(())
}
