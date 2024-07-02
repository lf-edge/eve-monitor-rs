use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use crate::traits::Component;

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl Component for Label {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        let text = Paragraph::new(self.text.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        frame.render_widget(text, *area);
    }
}
