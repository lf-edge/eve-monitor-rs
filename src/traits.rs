use ratatui::{layout::Rect, Frame};

use crate::{events::Event, ui::window::WindowId};

pub trait Component {
    // return a list of (parent, child) window ids
    fn get_children(&self) -> Vec<(WindowId, WindowId)> {
        vec![]
    }
    fn id(&self) -> WindowId;
    fn focused(&self) -> bool;
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn focus(&mut self);
    fn focus_lost(&mut self);
    fn name(&self) -> &str;
    // fn layout(&mut self, area: &Rect);
}

pub trait VisualComponent: Component {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, parent_focused: bool);
    // fn layout(&mut self, area: &Rect);
    fn handle_event(&mut self, _event: &Event) -> Option<Event> {
        // self.get_children()
        //     .and_then(|children| {
        //         for (parent, child) in children {
        //             if parent == self.id() {
        //                 return Some(child);
        //             }
        //         }
        //         None
        //     })
        //     .map(|id| Event::new(Event::Focus(id)));
        None
    }
    fn layout(&mut self, _area: &Rect) {}
}

impl std::fmt::Debug for dyn VisualComponent + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VisualComponent")
    }
}

// implement Debug for dyn traits::Component + 'static
impl std::fmt::Debug for dyn Component + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component")
    }
}
