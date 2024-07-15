use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, WidgetRef},
};

use crate::traits::{IEventHandler, IWidget};

use super::element::{IStandardRenderer, StaticElement};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWidgetState {
    text: String,
}

pub type LabelElement = StaticElement<LabelWidgetState>;

impl LabelElement {
    pub fn new<S: Into<String>>(text: S) -> Self {
        let state = LabelWidgetState { text: text.into() };
        let mut ret = Self {
            d: state,
            v: Default::default(),
            phantom: Default::default(),
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

impl IEventHandler for LabelElement {
    type Action = ();
}
impl IWidget for LabelElement {}
