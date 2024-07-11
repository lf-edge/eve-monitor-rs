use crate::events::{Event, UiCommand};
use ratatui::{layout::Rect, Frame};
use std::collections::HashMap;

pub trait IPresenter
where
    Self: IVisible + IFocusAcceptor,
{
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
    fn set_visible(&mut self, _visible: bool) {}
}

pub trait IFocusAcceptor {
    fn set_focus(&mut self) {}
    fn clear_focus(&mut self) {}
    fn has_focus(&self) -> bool {
        false
    }
    fn can_focus(&self) -> bool {
        true
    }
}

pub trait IFocusTracker {
    fn focus_next(&mut self) -> Option<String> {
        None
    }
    fn focus_prev(&mut self) -> Option<String> {
        None
    }
    fn get_focused_view_name(&self) -> Option<String> {
        None
    }
}

pub trait IEventHandler {
    fn handle_key_event(&mut self, _key: crossterm::event::KeyEvent) -> Option<Event> {
        None
    }
}

pub trait IEventDispatcher {
    fn dispatch_event(&self, event: UiCommand);
}

pub trait IWidgetPresenter {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>);
}

pub trait IWindow: IPresenter + IFocusTracker + IEventHandler + IEventDispatcher {}
pub trait IWidget: IWidgetPresenter + IEventHandler + IFocusAcceptor {}
