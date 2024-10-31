mod actions;
mod application;
mod events;
mod ipc;
mod model;
mod terminal;
mod traits;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use application::Application;
use libc::{EXIT_FAILURE, EXIT_SUCCESS};
use log::{info, LevelFilter};
use terminal::TerminalWrapper;

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

pub fn initialize_panic_handler() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .display_location_section(true)
        .display_env_section(true)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = TerminalWrapper::close_terminal();

        let msg = format!("{}", panic_hook.panic_report(panic_info));
        #[cfg(not(debug_assertions))]
        {
            eprintln!("{msg}");
            use human_panic::{handle_dump, print_msg, Metadata};
            let author = format!("authored by {}", env!("CARGO_PKG_AUTHORS"));
            let support = format!(
                "You can open a support request at {}",
                env!("CARGO_PKG_REPOSITORY")
            );
            let meta = Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .authors(author)
                .support(support);

            let file_path = handle_dump(&meta, panic_info);
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }
        log::error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(EXIT_FAILURE);
    }));
    Ok(())
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
    initialize_panic_handler()?;
    log_system_info();

    let mut app = Application::new()?;
    let result = app.run().await;
    if let Err(e) = &result {
        log::error!("Application error: {}", e);
    }
    // FIXME: this is a workaround for malfunctioning terminal event stream
    // Terminal must be dropped and restored automatically but one of the threads doesn't exit
    // and await? on a mina function never finishes. Drops are executed later.
    TerminalWrapper::close_terminal()?;
    std::process::exit(EXIT_SUCCESS);
}
