mod application;
mod dispatcher;
mod events;
mod mainwnd;
mod terminal;
mod traits;
mod ui;

use anyhow::{Ok, Result};
use application::Application;
use log::LevelFilter;
use pretty_env_logger::{self, formatted_builder};

fn init_logging() {
    formatted_builder().filter(None, LevelFilter::Trace).init();
}

fn main() -> Result<()> {
    println!("Hello, world!");
    init_logging();
    let mut app = Application::new()?;
    app.run()?;
    Ok(())
}
