// use std::collections::HashMap;

// use ratatui::{
//     buffer::Buffer,
//     layout::{Alignment, Position, Rect},
//     style::{Color, Style},
//     widgets::{Block, BorderType, Borders, Paragraph, StatefulWidgetRef, Widget},
//     Frame,
// };

// use crate::{
//     events::{Event, EventCode},
//     traits::{Component, VisualComponent},
// };

// use super::{
//     component::{StatefulComponentWrapper, VisualComponentState, WidgetState},
//     window::{Window, WindowId},
// };

// pub struct InputFieldWidget {}
// impl InputFieldWidget {
//     fn render_widget(
//         &self,
//         state: &mut VisualComponentState<InputFieldState>,
//         area: Rect,
//         buf: &mut Buffer,
//     ) {
//         let blk = Block::new()
//             .border_type(BorderType::Rounded)
//             .borders(Borders::ALL)
//             .border_style(Style::default().fg(Color::White))
//             .style(Style::default().bg(Color::Black))
//             .title(state.widget_state.caption.as_str());

//         // get inner area
//         let inner_area = blk.inner(area);
//         // render the border and caption
//         blk.render(area, buf);
//         // render the input field
//         let input = Paragraph::new(state.widget_state.value.as_deref().unwrap_or_default())
//             .style(Style::default().fg(Color::White))
//             .alignment(Alignment::Left);
//         input.render(inner_area, buf);
//         // set cursor position. It is set in global coordinates and it can be done only
//         // during rendering
//         state.widget_state.cursor_position = Position::new(
//             inner_area.x + state.widget_state.input_position as u16,
//             inner_area.y,
//         );
//     }
// }
// #[derive(Debug, Clone, PartialEq)]
// pub struct InputFieldState {
//     caption: String,
//     value: Option<String>,
//     input_position: usize,
//     cursor_position: Position,
// }

// impl WidgetState for InputFieldState {
//     fn get_layout(&self) -> std::collections::HashMap<String, Rect> {
//         todo!()
//     }
// }

// pub type InputField = StatefulComponentWrapper<InputFieldWidget, InputFieldState>;

// impl InputField {
//     pub fn new<S: Into<String>>(caption: S, value: Option<String>) -> Self {
//         let input_position = value.as_ref().map(|v| v.len()).unwrap_or_default();
//         let caption = caption.into();
//         Self::create_component_state(
//             caption.clone(),
//             Box::new(InputFieldWidget {}),
//             InputFieldState {
//                 caption,
//                 value,
//                 input_position,
//                 cursor_position: Position::new(0, 0),
//             },
//             Box::new(|_, _| HashMap::new()),
//             None,
//         )
//     }
// }

// impl StatefulWidgetRef for InputFieldWidget {
//     type State = VisualComponentState<InputFieldState>;
//     fn render_ref(
//         &self,
//         area: Rect,
//         buf: &mut Buffer,
//         state: &mut VisualComponentState<InputFieldState>,
//     ) {
//         self.render_widget(state, area, buf);
//     }
// }

// impl StatefulWidgetRef for &mut Box<InputFieldWidget> {
//     type State = VisualComponentState<InputFieldState>;
//     fn render_ref(
//         &self,
//         area: Rect,
//         buf: &mut Buffer,
//         state: &mut VisualComponentState<InputFieldState>,
//     ) {
//         self.render_widget(state, area, buf);
//     }
// }

// impl VisualComponent for InputField {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
//         frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
//         if self.state.focused() {
//             frame.set_cursor(
//                 self.state.widget_state.cursor_position.x,
//                 self.state.widget_state.cursor_position.y,
//             );
//         }
//     }
//     fn handle_event(&mut self, event: &EventCode) -> Option<Event> {
//         let old_state = self.state.widget_state.clone();

//         match event {
//             EventCode::Key(key) => match key.code {
//                 crossterm::event::KeyCode::Char(c) => {
//                     if let Some(value) = self.state.widget_state.value.as_mut() {
//                         value.insert(self.state.widget_state.input_position, c);
//                         self.state.widget_state.input_position += 1;
//                     }
//                 }
//                 crossterm::event::KeyCode::Backspace => {
//                     if let Some(value) = &mut self.state.widget_state.value {
//                         if self.state.widget_state.input_position > 0 {
//                             value.remove(self.state.widget_state.input_position - 1);
//                             self.state.widget_state.input_position -= 1;
//                         }
//                     }
//                 }
//                 crossterm::event::KeyCode::Delete => {
//                     if let Some(value) = &mut self.state.widget_state.value {
//                         if self.state.widget_state.input_position < value.len() {
//                             value.remove(self.state.widget_state.input_position);
//                         }
//                     }
//                 }
//                 crossterm::event::KeyCode::Left => {
//                     if self.state.widget_state.input_position > 0 {
//                         self.state.widget_state.input_position -= 1;
//                     }
//                 }
//                 crossterm::event::KeyCode::Right => {
//                     if let Some(value) = &self.state.widget_state.value {
//                         if self.state.widget_state.input_position < value.len() {
//                             self.state.widget_state.input_position += 1;
//                         }
//                     }
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//         if old_state != self.state.widget_state {
//             return Some(Event::app_event(EventCode::Redraw));
//         } else {
//             return None;
//         }
//     }

//     fn can_focus(&self) -> bool {
//         true
//     }
// }

use crossterm::{
    event::{KeyCode, KeyEvent},
    style,
};
use log::trace;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::{
    events::{Event, UiCommand},
    traits::{
        IEventHandler, IFocusAcceptor, IPresenter, IStatefulWidgetPresenter, IVisible, IWidget,
        IWidgetPresenter,
    },
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

impl InputFieldElement {
    pub fn new<S: Into<String>>(caption: S, value: Option<String>) -> Self {
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
    fn render_input_with_state(&self, area: Rect, buf: &mut Buffer, state: &mut InputFieldState) {
        let style = if self.has_focus() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let blk = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(style)
            .style(Style::default().bg(Color::Black))
            .title(self.d.caption.as_str());

        // get inner area
        let inner_area = blk.inner(area);
        // render the border and caption
        blk.render(area, buf);
        // render the input field
        let input = Paragraph::new(self.d.value.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        input.render(inner_area, buf);
        // set cursor position. It is set in global coordinates and it can be done only
        // during rendering
        state.cursor_position =
            Position::new(inner_area.x + self.d.input_position as u16, inner_area.y);
    }
}

impl IStatefulWidgetPresenter for InputFieldElement {
    type State = InputFieldState;
    fn render_with_state(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_input_with_state(area, buf, state);
    }
}

impl IStatefulWidgetPresenter for &mut InputFieldElement {
    type State = InputFieldState;
    fn render_with_state(&self, area: Rect, buf: &mut Buffer, state: &mut InputFieldState) {
        self.render_input_with_state(area, buf, state);
    }
}
impl IWidget for InputFieldElement {}
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
                    // if self.d.input_position > 0 {
                    //     self.d.input_position -= 1;
                    // }
                }
                KeyCode::Right => {
                    if self.d.input_position < value.len() {
                        self.d.input_position += 1;
                    }
                }
                KeyCode::Enter => todo!(),
                KeyCode::End => {
                    self.d.input_position = value.len();
                }
                KeyCode::Home => {
                    self.d.input_position = 0;
                }
                KeyCode::Tab => todo!(),
                KeyCode::BackTab => todo!(),
                KeyCode::Insert => {
                    self.d.input_mode.toggle();
                }
                KeyCode::Esc => todo!(),
                _ => {}
            }
            if old_state != self.d {
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
        }
        None
    }
}
impl IFocusAcceptor for InputFieldElement {
    fn set_focus(&mut self) {
        self.v.focused = true;
    }

    fn clear_focus(&mut self) {
        self.v.focused = false;
    }

    fn has_focus(&self) -> bool {
        self.v.focused
    }
}
impl IVisible for InputFieldElement {
    fn is_visible(&self) -> bool {
        self.v.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.v.visible = visible;
    }
}

impl IPresenter for InputFieldElement {
    fn do_layout(&mut self, area: &Rect) -> std::collections::HashMap<String, Rect> {
        todo!()
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        trace!("rendering: InputFieldElement {:#?}", &self);
        let mut state = self.d.clone();
        let is_focused = self.v.focused;
        frame.render_stateful_widget_ref(self, *area, &mut state);
        if is_focused {
            let pos = state.cursor_position;
            frame.set_cursor(pos.x, pos.y);
        }
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
