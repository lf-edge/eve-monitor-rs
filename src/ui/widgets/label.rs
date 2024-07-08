use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{self, Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, StatefulWidgetRef, WidgetRef},
    Frame,
};

use crate::traits::{IEventHandler, IPresenter, IWidget, IWidgetPresenter};

use super::element::Element;

pub struct LabelWidgetState {
    text: String,
}

pub type LabelElement = Element<LabelWidgetState>;

impl LabelElement {
    pub fn new(text: String) -> Self {
        let state = LabelWidgetState { text };
        Self {
            d: state,
            v: Default::default(),
        }
    }
}

impl IWidgetPresenter for LabelElement {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = self.d.text.clone();
        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        p.render_ref(area, buf);
    }
}

impl IWidgetPresenter for &mut LabelElement {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = self.d.text.clone();
        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        p.render_ref(area, buf);
    }
}

impl IPresenter for LabelElement {
    fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect> {
        todo!()
    }

    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        frame.render_widget_ref(self, *area)
    }
}

impl IEventHandler for LabelElement {}
impl IWidget for LabelElement {}
