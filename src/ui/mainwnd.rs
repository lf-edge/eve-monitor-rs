use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::traits::{Component, VisualComponent};

use super::{
    input_field::InputField,
    statusbar::StatusBarWidget,
    window::{Window, WindowId},
};

pub struct MainWnd {
    status_bar: Box<StatusBarWidget>,
    input_field: Box<InputField>,
    id: WindowId,
}

impl MainWnd {
    pub fn new() -> Self {
        Self {
            status_bar: Box::new(StatusBarWidget::new()),
            input_field: Box::new(InputField::new(
                "Input tech here".to_string(),
                Some("Input field".to_string()),
            )),
            id: Window::gen_window_id(),
        }
    }
}

impl VisualComponent for MainWnd {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, focused: bool) {
        use Constraint::{Length, Min};
        let [main_area, status_bar_area] = Layout::vertical([Min(0), Length(3)]).areas(*area);
        self.input_field.render(&main_area, frame, focused);
        self.status_bar.render(&status_bar_area, frame, focused);
    }
}
