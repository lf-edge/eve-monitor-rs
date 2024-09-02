use crossterm::event::KeyEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    TerminalResize(u16, u16),
}
