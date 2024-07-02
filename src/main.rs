mod application;
mod dispatcher;
mod events;
mod terminal;
mod traits;
mod ui;

use anyhow::{Ok, Result};
use application::Application;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut app = Application::new()?;
    app.run()?;
    Ok(())
}
