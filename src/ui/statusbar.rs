use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, WidgetRef},
};

pub struct StatusBar<'a> {
    border: Box<Block<'a>>,
}

impl<'a> StatusBar<'a> {
    pub fn new() -> Self {
        let border = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black));
        Self {
            border: Box::new(border),
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf)
    }
}

impl<'a> WidgetRef for StatusBar<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.border.render_ref(area, buf);
        // get current time in HH:MM:SS format
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        // and reneder it on the right
        let time = Paragraph::new(time_str.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        // split inner area of border into two parts
        let layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(time_str.len() as u16),
        ])
        .horizontal_margin(1)
        .split(self.border.inner(area));
        time.render(layout[1], buf);
    }
}
