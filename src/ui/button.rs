use log::trace;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols::block,
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidgetRef, Widget, WidgetRef},
};

use crate::{
    events::{Event, EventCode},
    traits::{Component, VisualComponent},
};

use super::{
    component::{StatefulComponentWrapper, WidgetState},
    window::{Window, WindowId},
};

pub struct ButtonWidgetState {
    label: String,
    pushed: bool,
    focused: bool,
}
impl ButtonWidgetState {
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
}

impl WidgetState for ButtonWidgetState {}

pub struct ButtonWidget {}
impl ButtonWidget {
    fn render(&self, state: &mut ButtonWidgetState, area: Rect, buf: &mut Buffer) {
        // set border style based on focus
        let border_style = if state.focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        // set border type based on push state
        let border_type = if state.focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .style(Style::default().bg(Color::Black));

        let button = if state.pushed {
            Paragraph::new(state.label.as_str())
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        } else {
            Paragraph::new(state.label.as_str())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        };
        button.render_ref(area, buf);
    }
}

pub type Button = StatefulComponentWrapper<ButtonWidget, ButtonWidgetState>;

impl Button {
    pub fn new(label: String) -> Self {
        Self::create_component_state(
            Box::new(ButtonWidget {}),
            Box::new(ButtonWidgetState {
                label,
                pushed: false,
                focused: false,
            }),
        )
    }
    pub fn label(&self) -> &str {
        self.state.label.as_str()
    }
}

impl StatefulWidgetRef for ButtonWidget {
    type State = ButtonWidgetState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut ButtonWidgetState) {
        self.render(state, area, buf);
    }
}

impl StatefulWidgetRef for &mut Box<ButtonWidget> {
    type State = ButtonWidgetState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut ButtonWidgetState) {
        self.render(state, area, buf);
    }
}

impl VisualComponent for Button {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>, _focused: bool) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
    }
    fn handle_event(&mut self, event: &Event) -> Option<Event> {
        match event.code {
            EventCode::Key(key) => {
                if self.state.pushed {
                    // we cate only about release of enter key or space bar
                    // consume all other events
                    if (key.code == crossterm::event::KeyCode::Enter
                        || key.code == crossterm::event::KeyCode::Char(' '))
                        && key.kind == crossterm::event::KeyEventKind::Release
                    {
                        self.state.pushed = false;
                        return Some(Event::redraw(None));
                    }
                    return None;
                } else {
                    if (key.code == crossterm::event::KeyCode::Enter
                        || key.code == crossterm::event::KeyCode::Char(' '))
                        && key.kind == crossterm::event::KeyEventKind::Press
                    {
                        self.state.pushed = true;
                        return Some(Event::redraw(None));
                    }
                    return None;
                }
            }
            _ => None,
        }
    }
}
