use crate::ui::action::UiActions;
use crossterm::event::KeyEvent;

pub enum Activity {
    Action(UiActions),
    Event(KeyEvent),
}

impl Activity {
    pub fn ui_action(action: UiActions) -> Self {
        Activity::Action(action)
    }

    pub fn key_event(key: KeyEvent) -> Self {
        Activity::Event(key)
    }

    pub fn redraw() -> Self {
        Activity::Action(UiActions::Redraw)
    }
}
