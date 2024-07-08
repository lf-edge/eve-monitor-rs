// // use crate::traits::TranslateEvent;

// // use crate::ui::window::{WindowId, TARGET_APP_ID};

// #[derive(Debug, Clone, PartialEq)]
// pub enum EventCode {
//     Key(crossterm::event::KeyEvent),
//     Tab,
//     ShiftTab,
//     Redraw,
//     Quit,
// }
// type WindowId = usize;
// const TARGET_APP_ID: WindowId = 0;
// #[derive(Debug, Clone)]
// pub struct Event {
//     pub code: EventCode,
//     pub target: Option<WindowId>,
// }

// impl Event {
//     pub fn new(code: EventCode, target: Option<WindowId>) -> Self {
//         Self { code, target }
//     }
//     pub fn app_event(code: EventCode) -> Self {
//         Self::new(code, Some(TARGET_APP_ID))
//     }
//     pub fn redraw(target: Option<WindowId>) -> Self {
//         Self::new(EventCode::Redraw, target.or(Some(TARGET_APP_ID)))
//     }
//     pub fn key_event(key: crossterm::event::KeyEvent) -> Self {
//         Self::new(EventCode::Key(key), None)
//     }
// }

// impl From<crossterm::event::Event> for EventCode {
//     fn from(key: crossterm::event::Event) -> Self {
//         match key {
//             crossterm::event::Event::Key(key) => EventCode::Key(key),
//             _ => EventCode::Redraw,
//         }
//     }
// }

// impl From<&crossterm::event::Event> for EventCode {
//     fn from(key: &crossterm::event::Event) -> Self {
//         match key {
//             crossterm::event::Event::Key(key) => EventCode::Key(key.clone()),
//             _ => EventCode::Redraw,
//         }
//     }
// }

// // impl TranslateEvent for crossterm::event::Event {
// //     //type Event = T;
// //     type Target = Event;

// //     fn translate_event(&self) -> Self::Target {
// //         Event::from(self)
// //     }
// // }

use crossterm::event::KeyEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    UiCommand(UiCommand),
}
#[derive(Clone, Debug)]
pub enum UiCommand {
    Redraw,
    Quit,
}

// impl From<crossterm::event::Event> for Event {
//     fn from(event: crossterm::event::Event) -> Self {
//         match event {
//             crossterm::event::Event::Key(key) => Event::Key(key),
//             _ => Err("Not a key event"),
//         }
//     }
// }
