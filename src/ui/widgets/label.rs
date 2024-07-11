use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, WidgetRef},
    Frame,
};

use crate::traits::{IEventHandler, IWidget, IWidgetPresenter};

use super::element::{Element, IStandardRenderer};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWidgetState {
    text: String,
}

pub type LabelElement = Element<LabelWidgetState>;

impl LabelElement {
    pub fn new<S: Into<String>>(text: S) -> Self {
        let state = LabelWidgetState { text: text.into() };
        let mut ret = Self {
            d: state,
            v: Default::default(),
        };
        ret.v.can_focus = false;
        ret
    }
}

impl IStandardRenderer for LabelElement {
    fn render(&self, area: &Rect, buf: &mut Buffer) {
        let text = self.d.text.clone();
        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        p.render_ref(*area, buf);
    }
}

impl IEventHandler for LabelElement {}
impl IWidget for LabelElement {}
