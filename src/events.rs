// use crate::traits::TranslateEvent;

#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Tab,
    Redraw,
}

impl From<crossterm::event::Event> for Event {
    fn from(key: crossterm::event::Event) -> Self {
        match key {
            crossterm::event::Event::Key(key) => Event::Key(key),
            _ => Event::Redraw,
        }
    }
}

impl From<&crossterm::event::Event> for Event {
    fn from(key: &crossterm::event::Event) -> Self {
        match key {
            crossterm::event::Event::Key(key) => Event::Key(key.clone()),
            _ => Event::Redraw,
        }
    }
}

// impl TranslateEvent for crossterm::event::Event {
//     //type Event = T;
//     type Target = Event;

//     fn translate_event(&self) -> Self::Target {
//         Event::from(self)
//     }
// }
