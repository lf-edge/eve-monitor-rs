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
    thread,
};

use ratatui::{backend::CrosstermBackend, CompletedFrame, Frame, Terminal};

use crate::{dispatcher::EventDispatcher, events::EventCode};

pub type IO = std::io::Stdout;

#[derive(Debug)]
pub struct TerminalWrapper {
    terminal: Terminal<CrosstermBackend<IO>>,
    terminal_thread_handle: thread::JoinHandle<Result<()>>,
}

impl TerminalWrapper {
    pub fn new(dispatcher: EventDispatcher<EventCode>) -> Result<Self> {
        let terminal = Self::init_terminal()?;
        let dispatcher = dispatcher.clone();
        // spawn a thread to listen for events
        let terminal_thread_handle = thread::spawn(move || -> Result<()> {
            loop {
                // wait for an event
                // if event is received, send it to the event dispatcher
                let event = crossterm::event::read()?;
                dispatcher.send(event.into());
            }
        });
        Ok(Self {
            terminal,
            terminal_thread_handle,
        })
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

    // wrapper to call draw on termnial
    fn draw<F>(&mut self, f: F) -> Result<CompletedFrame>
    where
        F: FnOnce(&mut Frame),
    {
        Ok(self.terminal.draw(f)?)
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
