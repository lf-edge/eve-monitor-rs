use crossterm::event::{KeyCode, KeyEvent};
use log::trace;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::{
    events::{Event, UiCommand},
    traits::{IEventHandler, IFocusAcceptor, IWidget, IWidgetPresenter},
};

use super::element::Element;
#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Insert,
    Overwrite,
}

impl InputMode {
    pub fn toggle(&mut self) {
        match self {
            Self::Insert => *self = Self::Overwrite,
            Self::Overwrite => *self = Self::Insert,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputFieldState {
    caption: String,
    value: Option<String>,
    input_position: usize,
    cursor_position: Position,
    input_mode: InputMode,
}

pub type InputFieldElement = Element<InputFieldState>;
impl IWidget for InputFieldElement {}

impl InputFieldElement {
    pub fn new<S: Into<String>>(caption: S, value: Option<S>) -> Self {
        let value = value.map(|v| v.into());
        let input_position = value.as_ref().map(|v| v.len()).unwrap_or_default();

        let caption = caption.into();
        Self {
            d: InputFieldState {
                caption,
                value,
                input_position,
                cursor_position: Position::new(0, 0),
                input_mode: InputMode::Insert,
            },
            v: Default::default(),
        }
    }
    fn render_input_field(&mut self, area: &Rect, buf: &mut Buffer) {
        let style = if self.has_focus() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let data = &mut self.d;

        let blk = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(style)
            .style(Style::default().bg(Color::Black))
            .title(data.caption.as_str());

        // get inner area
        let inner_area = blk.inner(*area);
        // render the border and caption
        blk.render(*area, buf);
        // render the input field
        let input = Paragraph::new(data.value.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        input.render(inner_area, buf);
        // set cursor position. It is set in global coordinates and it can be done only
        // during rendering
        self.d.cursor_position =
            Position::new(inner_area.x + self.d.input_position as u16, inner_area.y);
    }
}

impl IEventHandler for InputFieldElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Event> {
        let old_state = self.d.clone();
        if let Some(value) = self.d.value.as_mut() {
            match key.code {
                KeyCode::Char(c) => {
                    if self.d.input_mode == InputMode::Overwrite {
                        if self.d.input_position < value.len() {
                            value.remove(self.d.input_position);
                        }
                    }
                    value.insert(self.d.input_position, c);
                    self.d.input_position += 1;
                }
                KeyCode::Backspace => {
                    if self.d.input_position > 0 {
                        value.remove(self.d.input_position - 1);
                        self.d.input_position -= 1;
                    }
                }
                KeyCode::Delete => {
                    if self.d.input_position < value.len() {
                        value.remove(self.d.input_position);
                    }
                }
                KeyCode::Left => {
                    self.d.input_position = self.d.input_position.saturating_sub(1)
                    // if data.input_position > 0 {
                    //     data.input_position -= 1;
                    // }
                }
                KeyCode::Right => {
                    if self.d.input_position < value.len() {
                        self.d.input_position += 1;
                    }
                }
                KeyCode::Enter => {
                    // do nothing for now
                    // TODO: submit the value ?
                }
                KeyCode::End => {
                    self.d.input_position = value.len();
                }
                KeyCode::Home => {
                    self.d.input_position = 0;
                }
                KeyCode::Tab => {}
                KeyCode::BackTab => {}
                KeyCode::Insert => {
                    self.d.input_mode.toggle();
                }
                KeyCode::Esc => {}
                _ => {}
            }
            if old_state != self.d {
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
        }
        None
    }
}

impl IWidgetPresenter for InputFieldElement {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        trace!("rendering: InputFieldElement {:#?}", &self);
        self.render_input_field(area, frame.buffer_mut());

        // set cursor position must be called every time to display the cursor
        // on the next redraw cycle
        if self.has_focus() {
            let pos = self.d.cursor_position;
            frame.set_cursor(pos.x, pos.y);
        }
    }
}
