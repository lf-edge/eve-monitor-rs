use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidgetRef, Widget, WidgetRef},
};

use crate::traits::VisualComponent;

use super::component::{StatefulComponent, VisualComponentState, WidgetState};

pub struct StatusBarWidget {}
impl StatusBarWidget {
    fn render_widget(&self, _state: &mut StatusBarWidgetState, area: Rect, buf: &mut Buffer) {
        let border = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black));

        border.render_ref(area, buf);
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
        .split(border.inner(area));
        time.render(layout[1], buf);
    }
}

impl WidgetState for StatusBarWidgetState {
    fn get_layout(&self) -> std::collections::HashMap<String, Rect> {
        todo!()
    }
}

pub struct StatusBarWidgetState {}

impl StatefulWidgetRef for &mut Box<StatusBarWidget> {
    type State = StatusBarWidgetState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        self.render_widget(_state, area, buf);
    }
}

impl StatefulWidgetRef for StatusBarWidget {
    type State = VisualComponentState<StatusBarWidgetState>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_widget(&mut state.widget_state, area, buf);
    }
}

pub type StatusBar = StatefulComponent<StatusBarWidget, StatusBarWidgetState>;

impl StatusBar {
    pub fn new() -> Self {
        Self::create_component_state(
            "StatusBar".to_string(),
            Box::new(StatusBarWidget {}),
            StatusBarWidgetState {},
            Box::new(|_, _| HashMap::new()),
            None,
        )
    }
}

impl VisualComponent for StatusBar {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>, _focused: bool) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state.widget_state)
    }
}
