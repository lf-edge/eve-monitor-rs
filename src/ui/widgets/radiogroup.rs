use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use log::{info, trace};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, WidgetRef},
    Frame,
};

use crate::{
    traits::{IElementEventHandler, IFocusAcceptor, IWidget, IWidgetPresenter},
    ui::activity::Activity,
};

use super::element::VisualState;

pub struct RadioGroupElement {
    v: VisualState,
    pub labels: Vec<String>,
    pub selected: usize,
    pub focused: usize,
    pub title: String,
}

impl IWidget for RadioGroupElement {}

impl RadioGroupElement {
    pub fn new<S: Into<String>, P: Into<String>>(labels: Vec<S>, title: P) -> Self {
        Self {
            v: Default::default(),
            labels: labels.into_iter().map(|s| s.into()).collect(),
            selected: 0,
            focused: 0,
            title: title.into(),
        }
    }
    fn create_status_update(&self) -> Activity {
        info!("RadioGroupElement: selected: {}", self.selected);
        Activity::ui_action(crate::ui::action::UiActions::RadioGroup {
            selected: self.selected,
        })
    }
}

impl IWidgetPresenter for RadioGroupElement {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        //trace!("rendering: RadioGroupElement {:#?}", &self);
        let style = if self.has_focus() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(style);
        let inner = block.inner(*area);
        block.render_ref(*area, frame.buffer_mut());
        // create vertical layout for radio buttons
        let constraints = self.labels.iter().map(|_| Constraint::Length(1));
        let buttons_area = Layout::vertical(constraints).split(inner);

        let selected_style = Modifier::REVERSED;
        let normal_style = Style::default().fg(Color::White);

        // render paragraphs for each radio button
        for (i, label) in self.labels.iter().enumerate() {
            // format the button label <text> (selected)
            let mut style = normal_style;
            let label = if self.selected == i {
                format!("{} (*)", label)
            } else {
                format!("{} ( )", label)
            };

            if self.focused == i {
                style = style.add_modifier(selected_style);
            }

            let p = Paragraph::new(label).style(style);
            p.render_ref(buttons_area[i], frame.buffer_mut());
        }
    }
}

impl IElementEventHandler for RadioGroupElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Activity> {
        trace!("handle_key_event: RadioGroupView {}", &self.title);
        match key.code {
            KeyCode::Up => {
                self.focused = self.focused.saturating_sub(1);
                return Some(Activity::redraw());
            }
            KeyCode::Down => {
                self.focused = (self.focused + 1).min(self.labels.len() - 1);
                return Some(Activity::redraw());
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.selected = self.focused;
                return Some(self.create_status_update());
            }
            _ => {
                return None;
            }
        }
    }
}

impl IFocusAcceptor for RadioGroupElement {
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
