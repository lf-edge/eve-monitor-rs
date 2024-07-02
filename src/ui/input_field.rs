use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidgetRef, Widget},
    Frame,
};

use crate::{events::Event, traits::Component};

struct InputFieldWidget {}
#[derive(Debug, Clone, PartialEq)]
struct InputFieldState {
    caption: String,
    value: Option<String>,
    input_position: usize,
    cursor_position: Position,
}

pub struct InputField {
    widget: InputFieldWidget,
    state: InputFieldState,
}

impl InputField {
    pub fn new(caption: String, value: Option<String>) -> Self {
        let input_position = value.as_ref().map(|v| v.len()).unwrap_or_default();
        Self {
            widget: InputFieldWidget {},
            state: InputFieldState {
                caption,
                value,
                input_position,
                cursor_position: Position::new(0, 0),
            },
        }
    }
}

impl StatefulWidgetRef for &mut InputFieldWidget {
    type State = InputFieldState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut InputFieldState) {
        let blk = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(state.caption.as_str());

        // get inner area
        let inner_area = blk.inner(area);
        // render the border and caption
        blk.render(area, buf);
        // render the input field
        let input = Paragraph::new(state.value.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        input.render(inner_area, buf);
        // set cursor position. It is set in global coordinates and it can be done only
        // during rendering
        state.cursor_position =
            Position::new(inner_area.x + state.input_position as u16, inner_area.y);
    }
}

impl Component for InputField {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
        frame.set_cursor(self.state.cursor_position.x, self.state.cursor_position.y);
    }
    fn handle_event(&mut self, event: &Event) -> Option<Event> {
        let old_state = self.state.clone();

        match event {
            Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Char(c) => {
                    if let Some(value) = &mut self.state.value {
                        value.insert(self.state.input_position, c);
                        self.state.input_position += 1;
                    }
                }
                crossterm::event::KeyCode::Backspace => {
                    if let Some(value) = &mut self.state.value {
                        if self.state.input_position > 0 {
                            value.remove(self.state.input_position - 1);
                            self.state.input_position -= 1;
                        }
                    }
                }
                crossterm::event::KeyCode::Delete => {
                    if let Some(value) = &mut self.state.value {
                        if self.state.input_position < value.len() {
                            value.remove(self.state.input_position);
                        }
                    }
                }
                crossterm::event::KeyCode::Left => {
                    if self.state.input_position > 0 {
                        self.state.input_position -= 1;
                    }
                }
                crossterm::event::KeyCode::Right => {
                    if let Some(value) = &self.state.value {
                        if self.state.input_position < value.len() {
                            self.state.input_position += 1;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        if old_state != self.state {
            return Some(Event::Redraw);
        } else {
            return None;
        }
    }
}
