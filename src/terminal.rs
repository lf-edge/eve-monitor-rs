use anyhow::Result;
use crossterm::{
    cursor, execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    io::stdout,
    ops::{Deref, DerefMut},
};

use ratatui::{backend::CrosstermBackend, Terminal};

pub type IO = std::io::Stdout;

#[derive(Debug)]
pub struct TerminalWrapper {
    terminal: Terminal<CrosstermBackend<IO>>,
}

impl TerminalWrapper {
    pub fn new() -> Result<Self> {
        let terminal = Self::init_terminal()?;
        Ok(Self { terminal })
    }

    fn init_terminal() -> Result<Terminal<CrosstermBackend<IO>>> {
        println!("Initializing terminal");
        // No stdout after this point
        execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn close_terminal(&mut self) -> Result<()> {
        if is_raw_mode_enabled()? {
            execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
            disable_raw_mode()?;
        }
        // No stdout before this point
        println!("Terminal should be now closed");

        Ok(())
    }

    pub fn get_stream(&self) -> crossterm::event::EventStream {
        crossterm::event::EventStream::new()
    }
}

impl Drop for TerminalWrapper {
    fn drop(&mut self) {
        self.close_terminal().unwrap();
    }
}

impl Deref for TerminalWrapper {
    type Target = Terminal<CrosstermBackend<IO>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}
