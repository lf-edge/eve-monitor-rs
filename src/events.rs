use crossterm::event::KeyEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    UiCommand(UiCommand),
}

impl Event {
    pub fn redraw() -> Self {
        Event::UiCommand(UiCommand::Redraw)
    }
}

#[derive(Clone, Debug)]
pub enum UiCommand {
    Redraw,
    Quit,
}
