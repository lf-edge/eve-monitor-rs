use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    traits::{
        IEventHandler, IFocusAcceptor, IFocusTracker, ILayout, IPresenter, IVisible, IWidget,
    },
    ui::focus_tracker::FocusTracker,
};

use super::element::WidgetWithLayout;

#[derive(Debug)]
pub struct RadioGroupState {
    pub labels: Vec<String>,
    pub selected: usize,
    pub title: String,
    pub in_focus: bool,
    pub is_visible: bool,
}

impl ILayout for RadioGroupState {
    fn get_layout(&self) -> HashMap<String, ratatui::prelude::Rect> {
        todo!()
    }

    fn set_layout(&self, layout: HashMap<String, ratatui::prelude::Rect>) {
        todo!()
    }
}

impl IFocusAcceptor for RadioGroupState {
    fn set_focus(&mut self) {
        self.in_focus = true;
    }

    fn clear_focus(&mut self) {
        self.in_focus = false;
    }
}

impl IVisible for RadioGroupState {
    fn is_visible(&self) -> bool {
        self.is_visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
}

#[derive(Debug)]
pub struct RadioGroupWidget {}

impl StatefulWidgetRef for RadioGroupWidget {
    type State = RadioGroupState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        //let layout = state.get_layout();
        info!("rendering: RadioGroupWidget {:#?}", &state);
        let style = if state.in_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(state.title.clone())
            .borders(Borders::ALL)
            .border_style(style);
        let inner = block.inner(area);
        block.render_ref(area, buf);
        // create vertical layout for radio buttons
        let constraints = state.labels.iter().map(|_| Constraint::Length(1));
        let buttons_area = Layout::vertical(constraints).split(inner);

        // render paragraphs for each radio button
        for (i, label) in state.labels.iter().enumerate() {
            // format the button label <text> (selected)
            let label = if state.selected == i {
                format!("{} (*)", label)
            } else {
                format!("{} ( )", label)
            };

            let p = Paragraph::new(label);
            p.render_ref(buttons_area[i], buf);
        }
    }
}

#[derive(Debug)]
pub struct RadioGroupView {
    pub state: RadioGroupState,
    pub widget: WidgetWithLayout<RadioGroupWidget>,
    pub ft: FocusTracker,
}

impl IFocusAcceptor for RadioGroupView {
    fn set_focus(&mut self) {
        self.state.set_focus();
    }

    fn clear_focus(&mut self) {
        self.state.clear_focus();
    }
}

impl IVisible for RadioGroupView {
    fn is_visible(&self) -> bool {
        self.state.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.state.set_visible(visible);
    }
}

impl IPresenter for RadioGroupView {
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
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
        // self.widget
        //     .render_ref(*area, frame.buffer_mut(), &mut self.state)
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}

impl IFocusTracker for RadioGroupView {
    fn focus_next(&mut self) -> Option<&String> {
        self.ft.focus_next()
    }

    fn focus_prev(&mut self) -> Option<&String> {
        self.ft.focus_prev()
    }

    fn get_focused_view_name(&self) -> Option<&String> {
        Some(&self.state.labels[self.state.selected])
    }
}

impl IEventHandler for RadioGroupView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        info!("handle_key_event: RadioGroupView {:#?}", &self);
        //TODO: change to focus tracker
        match key.code {
            KeyCode::Up => {
                self.state.selected = self.state.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.state.selected = (self.state.selected + 1).min(self.state.labels.len() - 1);
            }
            _ => {}
        }
    }
}
impl IWidget for RadioGroupView {}
