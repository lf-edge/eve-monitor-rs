mod actions;
mod application;
mod device;
mod events;
mod ipc;
mod mainwnd;
mod model;
mod raw_model;
mod terminal;
mod traits;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use application::Application;
use log::{info, LevelFilter};

fn init_logging() -> log2::Handle {
    let log_dir = if let Ok(_dir) = std::env::var("XDG_RUNTIME_DIR") {
        // store log in the current directory for convenience
        // we use XDG_RUNTIME_DIR to detect the fact that we are running on desktop linux
        PathBuf::from("./")
    } else {
        // get current data and time and use it as a log file name
        let now = chrono::Local::now();
        // make /persist/monitor-<date>-<time>/ folder path and create he folder
        let log_dir = PathBuf::from(format!(
            "/persist/monitor-{}",
            now.format("%Y-%m-%d-%H-%M-%S")
        ));
        std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");
        // set EVE_MONITOR_LOG_DIR to the created folder
        std::env::set_var("EVE_MONITOR_LOG_DIR", log_dir.to_string_lossy().to_string());

        log_dir
    };

    let log_file = log_dir.join("monitor.log").to_string_lossy().to_string();

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

fn log_system_info() {
    // get current user UID and GID
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self")
        .and_then(|m| {
            info!("Current process UID: {}, GID: {}", m.uid(), m.gid());
            Ok(())
        })
        .unwrap_or_else(|e| {
            info!("Failed to get current process UID and GID: {}", e);
        });
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log2 = init_logging();

    log_system_info();

    let mut app = Application::new()?;
    app.run().await?;
    Ok(())
}
