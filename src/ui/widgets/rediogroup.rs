use std::borrow::BorrowMut;

use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, WidgetRef},
    Frame,
};

use crate::{
    events::{Event, UiCommand},
    traits::{IEventHandler, IFocusAcceptor, IWidget, IWidgetPresenter},
};

use super::element::{Element, IStandardRenderer};

#[derive(Debug, Clone, PartialEq)]
pub struct RadioGroupState {
    pub labels: Vec<String>,
    pub selected: usize,
    pub title: String,
}

pub type RadioGroupElement = Element<RadioGroupState>;
impl IWidget for RadioGroupElement {}

impl RadioGroupElement {
    pub fn new<S: Into<String>, P: Into<String>>(labels: Vec<S>, title: P) -> Self {
        let state = RadioGroupState {
            labels: labels.into_iter().map(|s| s.into()).collect(),
            selected: 0,
            title: title.into(),
        };
        Self {
            d: state,
            v: Default::default(),
        }
    }
}

impl IStandardRenderer for RadioGroupElement {
    fn render(&self, area: &Rect, buf: &mut Buffer) {
        info!("rendering: RadioGroupWidget {:#?}", &self);
        let style = if self.has_focus() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(self.d.title.clone())
            .borders(Borders::ALL)
            .border_style(style);
        let inner = block.inner(*area);
        block.render_ref(*area, buf);
        // create vertical layout for radio buttons
        let constraints = self.d.labels.iter().map(|_| Constraint::Length(1));
        let buttons_area = Layout::vertical(constraints).split(inner);

        // render paragraphs for each radio button
        for (i, label) in self.d.labels.iter().enumerate() {
            // format the button label <text> (selected)
            let label = if self.d.selected == i {
                format!("{} (*)", label)
            } else {
                format!("{} ( )", label)
            };

            let p = Paragraph::new(label);
            p.render_ref(buttons_area[i], buf);
        }
    }
}

impl IEventHandler for RadioGroupElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Event> {
        info!("handle_key_event: RadioGroupView {:#?}", &self);
        //TODO: change to focus tracker
        match key.code {
            KeyCode::Up => {
                self.d.borrow_mut().selected = self.d.borrow_mut().selected.saturating_sub(1);
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
            KeyCode::Down => {
                self.d.borrow_mut().selected =
                    (self.d.borrow_mut().selected + 1).min(self.d.borrow_mut().labels.len() - 1);
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
            _ => {
                return None;
            }
        }
    }
}
