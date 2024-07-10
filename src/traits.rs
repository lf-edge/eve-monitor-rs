use crate::events::{Event, UiCommand};
use ratatui::{buffer::Buffer, layout::Rect, Frame};
use std::collections::HashMap;

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
    fn has_focus(&self) -> bool {
        false
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
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Option<Event> {
        None
    }
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

pub trait IStatefulWidgetPresenter {
    type State;
    fn render_with_state(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}

pub trait IWindow: IPresenter + IFocusTracker + IEventHandler + IEventDispatcher {}
pub trait IVisibleElement: IPresenter + IVisible + IFocusAcceptor {}
pub trait IWidget: IPresenter + IEventHandler {}

// pub trait IWithTabOrder: IWidget {
//     fn set_tab_order(&mut self, order: u32);
//     fn get_tab_order(&self) -> u32;
// }
