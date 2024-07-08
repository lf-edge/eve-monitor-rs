// use ratatui::{layout::Rect, Frame};

// use crate::{
//     events::{Event, EventCode},
//     ui::window::WindowId,
// };

// pub trait Component {
//     // return a list of (parent, child) window ids
//     fn get_children(&self) -> Vec<(WindowId, WindowId)> {
//         vec![]
//     }
//     fn id(&self) -> WindowId;
//     fn visible(&self) -> bool;
//     fn set_visible(&mut self, visible: bool);
//     fn focus_lost(&mut self);
//     fn focus(&mut self);
//     fn focus_next(&mut self) -> bool;

//     fn name(&self) -> &str;
//     fn get_view_mut(&mut self, name: &str) -> Option<&mut Box<dyn VisualComponent>>;
//     // fn layout(&mut self, area: &Rect);
// }

// pub trait VisualComponent: Component {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, parent_focused: bool);
//     // fn layout(&mut self, area: &Rect);
//     fn handle_event(&mut self, _event: &EventCode) -> Option<Event> {
//         // self.get_children()
//         //     .and_then(|children| {
//         //         for (parent, child) in children {
//         //             if parent == self.id() {
//         //                 return Some(child);
//         //             }
//         //         }
//         //         None
//         //     })
//         //     .map(|id| Event::new(Event::Focus(id)));
//         None
//     }
//     fn layout(&mut self, _area: &Rect) {}
//     fn can_focus(&self) -> bool {
//         false
//     }
//     fn get_view_mut(&mut self, name: &str) -> Option<&mut Box<dyn VisualComponent>> {
//         Component::get_view_mut(self, name)
//     }
// }

// impl std::fmt::Debug for dyn VisualComponent + 'static {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "VisualComponent")
//     }
// }

// // implement Debug for dyn traits::Component + 'static
// impl std::fmt::Debug for dyn Component + 'static {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Component")
//     }
// }

use std::collections::HashMap;

use crossterm::event::{Event, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect, Frame};

use crate::events::UiCommand;

pub trait IPresenter: IVisible + IFocusAcceptor {
    fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect>;
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>);
    fn is_focus_tracker(&self) -> bool {
        false
    }
}

pub trait IVisible {
    fn is_visible(&self) -> bool {
        true
    }
    fn set_visible(&mut self, visible: bool) {}
}

pub trait IFocusAcceptor {
    fn set_focus(&mut self) {}
    fn clear_focus(&mut self) {}
}



pub trait IFocusTracker {
    fn focus_next(&mut self) -> Option<&String> {
        None
    }
    fn focus_prev(&mut self) -> Option<&String> {
        None
    }
    fn get_focused_view_name(&self) -> Option<&String> {
        None
    }
}

pub trait IEventHandler {
    fn handle_key_event(&mut self, key: KeyEvent);
}

pub trait IEventDispatcher {
    fn dispatch_event(&self, event: UiCommand);
}

pub trait ILayout {
    fn get_layout(&self) -> HashMap<String, ratatui::prelude::Rect>;
    fn set_layout(&self, layout: HashMap<String, ratatui::prelude::Rect>);
}

// pub trait IWidgetPresenter {
//     fn render(&self, area: Rect, buf: &mut Buffer);
// }

pub trait IWidgetPresenter {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

pub trait IWindow: IPresenter + IFocusTracker + IEventHandler + IEventDispatcher {}
pub trait IVisibleElement: IPresenter + IVisible + IFocusAcceptor {}
pub trait IWidget: IPresenter + IEventHandler {}
