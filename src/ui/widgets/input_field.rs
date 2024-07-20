use crate::ui::action::Action;
use crate::ui::activity::Activity;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::{
    traits::{IElementEventHandler, IFocusAcceptor, IWidget, IWidgetPresenter},
    ui::action::UiActions,
};

use super::element::VisualState;
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

pub type OnContentUpdated = dyn FnMut(&String) -> Option<String>;
pub type OnChar = dyn FnMut(&char) -> Option<char>;

pub struct InputFieldElement {
    v: VisualState,
    caption: String,
    value: Option<String>,
    input_position: usize,
    cursor_position: Position,
    input_mode: InputMode,
    on_update: Option<Box<OnContentUpdated>>,
    on_char: Option<Box<OnChar>>,
}

impl IWidget for InputFieldElement {}

impl InputFieldElement {
    pub fn new<S: Into<String>>(caption: S, value: Option<S>) -> Self {
        let value = value.map(|v| v.into());
        let input_position = value.as_ref().map(|v| v.len()).unwrap_or_default();

        let caption = caption.into();
        Self {
            caption,
            value,
            input_position,
            cursor_position: Position::new(0, 0),
            input_mode: InputMode::Insert,
            v: Default::default(),
            on_update: None,
            on_char: None,
        }
    }

    pub fn on_update<F>(mut self, f: F) -> Self
    where
        F: Fn(&String) -> Option<String> + 'static,
    {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn on_char<F>(mut self, f: F) -> Self
    where
        F: FnMut(&char) -> Option<char> + 'static,
    {
        self.on_char = Some(Box::new(f));
        self
    }

    fn render_input_field(&mut self, area: &Rect, buf: &mut Buffer) {
        let style = if self.has_focus() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // let data = &mut self.caption;

        let blk = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(style)
            .style(Style::default().bg(Color::Black))
            .title(self.caption.clone());

        // get inner area
        let inner_area = blk.inner(*area);
        // render the border and caption
        blk.render(*area, buf);
        // render the input field
        let input = Paragraph::new(self.value.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        input.render(inner_area, buf);
        // set cursor position. It is set in global coordinates and it can be done only
        // during rendering
        self.cursor_position =
            Position::new(inner_area.x + self.input_position as u16, inner_area.y);
    }
}

impl IElementEventHandler for InputFieldElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Activity> {
        let old_value = self.value.clone();
        if let Some(value) = self.value.as_mut() {
            match key.code {
                KeyCode::Char(c) => {
                    if let Some(f) = self.on_char.as_mut() {
                        if let Some(c) = f(&c) {
                            if self.input_mode == InputMode::Overwrite {
                                if self.input_position < value.len() {
                                    value.remove(self.input_position);
                                }
                            }
                            value.insert(self.input_position, c);
                            self.input_position += 1;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if self.input_position > 0 {
                        value.remove(self.input_position - 1);
                        self.input_position -= 1;
                    }
                }
                KeyCode::Delete => {
                    if self.input_position < value.len() {
                        value.remove(self.input_position);
                    }
                }
                KeyCode::Left => {
                    self.input_position = self.input_position.saturating_sub(1)
                    // if data.input_position > 0 {
                    //     data.input_position -= 1;
                    // }
                }
                KeyCode::Right => {
                    if self.input_position < value.len() {
                        self.input_position += 1;
                    }
                }
                KeyCode::Enter => {
                    // do nothing for now
                    // TODO: submit the value ?
                }
                KeyCode::End => {
                    self.input_position = value.len();
                }
                KeyCode::Home => {
                    self.input_position = 0;
                }
                KeyCode::Tab => {}
                KeyCode::BackTab => {}
                KeyCode::Insert => {
                    self.input_mode.toggle();
                }
                KeyCode::Esc => {}
                _ => {}
            }
            if old_value != self.value {
                return Some(Activity::Action(UiActions::Input {
                    text: self.value.clone().unwrap_or_default(),
                }));
            }
        }
        None
    }
}

impl IWidgetPresenter for InputFieldElement {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>, focused: bool) {
        //trace!("rendering: InputFieldElement {:#?}", &self);
        self.render_input_field(area, frame.buffer_mut());

        // set cursor position must be called every time to display the cursor
        // on the next redraw cycle
        if focused {
            let pos = self.cursor_position;
            frame.set_cursor(pos.x, pos.y);
        }
    }
}

impl IFocusAcceptor for InputFieldElement {
    fn set_focus(&mut self, focus: bool) {
        self.v.focused = focus;
    }

    fn clear_focus(&mut self) {
        self.v.focused = false;
    }

    fn has_focus(&self) -> bool {
        self.v.focused
    }

    fn can_focus(&self) -> bool {
        true
    }
}
