mod actions;
mod application;
mod events;
mod mainwnd;
mod terminal;
mod traits;
mod ui;

use anyhow::{Ok, Result};
use application::Application;
use log::LevelFilter;
use pretty_env_logger::env_logger::WriteStyle;
use pretty_env_logger::formatted_builder;

fn init_logging() {
    formatted_builder()
        .filter(None, LevelFilter::Debug)
        .write_style(WriteStyle::Always)
        .init();
}
#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    init_logging();
    let mut app = Application::new()?;
    app.run().await?;
    Ok(())
}
