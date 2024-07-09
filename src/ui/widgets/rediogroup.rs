use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, WidgetRef},
};

use crate::{
    events::{Event, UiCommand},
    traits::{IEventHandler, IFocusAcceptor, IPresenter, IWidget, IWidgetPresenter},
};

use super::element::Element;

#[derive(Debug)]
pub struct RadioGroupState {
    pub labels: Vec<String>,
    pub selected: usize,
    pub title: String,
}

pub type RadioGroupElement = Element<RadioGroupState>;

impl RadioGroupElement {
    pub fn new(labels: Vec<String>, title: String) -> Self {
        let state = RadioGroupState {
            labels,
            selected: 0,
            title,
        };
        Self {
            d: state,
            v: Default::default(),
        }
    }
    pub fn render_view(&self, area: Rect, buf: &mut Buffer) {
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
        let inner = block.inner(area);
        block.render_ref(area, buf);
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

impl IWidgetPresenter for RadioGroupElement {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render_view(area, buf);
    }
}

impl IWidgetPresenter for &mut RadioGroupElement {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.render_view(area, buf);
    }
}

impl IPresenter for RadioGroupElement {
    fn do_layout(
        &mut self,
        area: &Rect,
    ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
        // let mut layout_map = std::collections::HashMap::new();
        // layout_map.insert("RadioGroup".to_string(), *area);

        // info!("do_layout: RadioGroupView {:#?}", &self);
        // return HashMap::new();
        todo!("RadioGroupView::do_layout")
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        info!("rendering: RadioGroupView {:#?}", &self);
        frame.render_widget_ref(self, *area);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}

impl IEventHandler for RadioGroupElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Event> {
        info!("handle_key_event: RadioGroupView {:#?}", &self);
        //TODO: change to focus tracker
        match key.code {
            KeyCode::Up => {
                self.d.selected = self.d.selected.saturating_sub(1);
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
            KeyCode::Down => {
                self.d.selected = (self.d.selected + 1).min(self.d.labels.len() - 1);
                return Some(Event::UiCommand(UiCommand::Redraw));
            }
            _ => {
                return None;
            }
        }
    }
}

impl IWidget for RadioGroupElement {
    // fn get_name(&self) -> &str {
    //     "RadioGroup"
    // }
}
