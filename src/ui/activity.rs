use crate::ui::action::UiActions;
use crossterm::event::KeyEvent;

pub enum Activity {
    Action(UiActions),
    Event(KeyEvent),
}
