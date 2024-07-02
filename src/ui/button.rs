use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols::block,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidgetRef, Widget, WidgetRef},
};

use crate::{events::Event, traits::Component};

pub struct ButtonState {
    label: String,
    pushed: bool,
}

struct ButtonWidget {}

pub struct Button {
    state: ButtonState,
    widget: ButtonWidget,
}

impl Button {
    pub fn new(label: String) -> Self {
        Self {
            state: ButtonState {
                label: label.clone(),
                pushed: false,
            },
            widget: ButtonWidget {},
        }
    }
    pub fn label(&self) -> &str {
        self.state.label.as_str()
    }
}

impl StatefulWidgetRef for &mut ButtonWidget {
    type State = ButtonState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut ButtonState) {
        let button = if state.pushed {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Black));

            Paragraph::new(state.label.as_str())
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Black));

            Paragraph::new(state.label.as_str())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        };
        button.render_ref(area, buf);
    }
}

impl Component for Button {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
    }
    fn handle_event(&mut self, event: &Event) -> Option<Event> {
        match event {
            Event::Key(key) => {
                if self.state.pushed {
                    // we cate only about release of enter key or space bar
                    // consume all other events
                    if (key.code == crossterm::event::KeyCode::Enter
                        || key.code == crossterm::event::KeyCode::Char(' '))
                        && key.kind == crossterm::event::KeyEventKind::Release
                    {
                        self.state.pushed = false;
                        return Some(Event::Redraw);
                    }
                    return None;
                } else {
                    if (key.code == crossterm::event::KeyCode::Enter
                        || key.code == crossterm::event::KeyCode::Char(' '))
                        && key.kind == crossterm::event::KeyEventKind::Press
                    {
                        self.state.pushed = true;
                        return Some(Event::Redraw);
                    }
                    return None;
                }
            }
            _ => None,
        }
    }
}
