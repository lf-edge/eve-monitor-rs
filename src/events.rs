use crossterm::event::KeyEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    TerminalResize(u16, u16),
}
