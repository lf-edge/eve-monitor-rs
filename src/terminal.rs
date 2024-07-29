use anyhow::Result;
use crossterm::{
    cursor, execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    fs::{self, File},
    io::stdout,
    ops::{Deref, DerefMut},
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd},
};

use ratatui::{backend::CrosstermBackend, Terminal};

#[derive(Debug)]
pub struct TerminalWrapper {
    terminal: Terminal<CrosstermBackend<File>>,
    fd: RawFd,
}

impl TerminalWrapper {
    fn tty_fd() -> Result<File> {
        Ok(fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?)
    }

    fn init_terminal(file: File) -> Result<Terminal<CrosstermBackend<File>>> {
        println!("Initializing terminal");
        // No stdout after this point
        execute!(&file, EnterAlternateScreen, cursor::Hide)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(file))?;
        terminal.clear()?;
        Ok(terminal)
    }

    pub fn open_terminal() -> Result<Self> {
        let file = Self::tty_fd()?;
        let raw_fd = file.as_raw_fd();

        let terminal = Self::init_terminal(file)?;
        Ok(Self {
            terminal,
            fd: raw_fd,
        })
    }

    fn close_terminal(&mut self) -> Result<()> {
        self.terminal.clear()?;
        if is_raw_mode_enabled()? {
            // get file from raw fd
            let mut file = unsafe { File::from_raw_fd(self.fd) };
            execute!(file, LeaveAlternateScreen, cursor::Show)?;
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
    type Target = Terminal<CrosstermBackend<File>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}
