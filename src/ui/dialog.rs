use std::collections::HashMap;

use crossterm::event::KeyEvent;
use log::{info, trace};
use ratatui::{
    layout::{self, Constraint, Flex, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
    Frame,
};

use crate::traits::{IEventHandler, IFocusAcceptor, IFocusTracker, IPresenter, IVisible, IWindow};

use anyhow::Result;

use super::{
    action::{Action, UiActions},
    tools::centered_rect_fixed,
    widgets::button::ButtonElement,
    window::{LayoutMap, WidgetMap, Window},
};

pub struct Dialog<A, D> {
    w: Window<A, ()>,
    size: (u16, u16),
    buttons: Vec<String>,
    state: D,
    layout: LayoutMap,
}

impl<A: 'static, D: 'static> Dialog<A, D> {
    pub fn new(size: (u16, u16), buttons: Vec<String>, focused_button: &str, state: D) -> Self {
        let mut w = Window::builder("Dialog")
            .with_layout(|_| None)
            .with_render(Self::do_render)
            .with_focused_view(focused_button)
            .with_state(());

        // create buttons and add them to the window builder
        for button_name in buttons.iter() {
            let button = ButtonElement::<A>::new(button_name);
            w = w.widget(button_name, Box::new(button));
        }

        Self {
            w: w.build().unwrap(),
            size,
            buttons,
            state,
            layout: LayoutMap::new(),
        }
    }

    fn on_ok_yes<F>(f: F) -> Option<UiActions<A>>
    where
        F: Fn(&D) -> Option<UiActions<A>>,
    {
        Some(UiActions::ButtonClicked("Ok".to_string()))
    }

    fn with_render(
        frame: &mut Frame<'_>,
        layout: &LayoutMap,
        widgets: &mut WidgetMap<A>,
    ) -> Result<()> {
        info!("Rendering dialog content");
        Ok(())
    }

    fn do_layout(&mut self, area: &Rect) {
        let dialog_area = centered_rect_fixed(self.size.0, self.size.1, *area);
        self.layout.insert("frame".to_string(), dialog_area);
        // split the dialog area into two parts: content and buttons
        let max_button_len = self.buttons.iter().map(|b| b.len() + 2).max().unwrap_or(0) as u16;
        let num_buttons = self.buttons.len();

        let layout = layout::Layout::horizontal([
            layout::Constraint::Min(0),
            layout::Constraint::Length(max_button_len),
        ])
        .margin(1)
        .split(dialog_area);

        let content_rect = layout[0];
        let buttons_rect = layout[1];

        // split the buttons area into buttons
        let button_layout = layout::Layout::vertical(vec![Constraint::Length(3); num_buttons])
            .flex(Flex::Start)
            .split(buttons_rect);

        for (i, button) in self.buttons.iter().enumerate() {
            self.layout.insert(button.clone(), button_layout[i]);
        }
        self.layout.insert("content".to_string(), content_rect);
    }

    fn do_render(
        _area: &Rect,
        frame: &mut Frame<'_>,
        layout: &LayoutMap,
        widgets: &mut WidgetMap<A>,
    ) {
        info!("Rendering dialog content");
    }
}

impl<A: 'static, D: 'static> IWindow for Dialog<A, D> {}

impl<A, D> IFocusTracker for Dialog<A, D> {
    fn focus_next(&mut self) -> Option<String> {
        self.w.focus_next()
    }

    fn focus_prev(&mut self) -> Option<String> {
        self.w.focus_prev()
    }

    fn get_focused_view_name(&self) -> Option<String> {
        self.w.get_focused_view_name()
    }
}

impl<A: 'static, D: 'static> IPresenter for Dialog<A, D> {
    // fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect> {
    //     self.do_layout(area);
    //     // get content area and pass it to window
    //     let content_area = self.layout.get("content").unwrap();

    //     self.w.do_layout(&content_area);
    //     HashMap::new()
    // }

    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        trace!("Rendering dialog: {}", self.w.name);
        self.do_layout(area);
        // render the dialog
        let frame_rect = self.layout.get("frame").unwrap();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.w.name.as_str());

        block.render(*frame_rect, frame.buffer_mut());
        // render the buttons
        for button_name in self.buttons.iter() {
            let button_rect = self.layout.get(button_name).unwrap();
            let button = self.w.widgets.get_mut(button_name).unwrap();
            button.render(button_rect, frame);
        }

        // render the content
        let content_area = self.layout.get("content").unwrap();
        self.w.render(content_area, frame);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}

impl<A, D> IFocusAcceptor for Dialog<A, D> {
    fn has_focus(&self) -> bool {
        // dialog is always focused
        true
    }

    fn set_focus(&mut self) {
        self.w.set_focus();
    }

    fn clear_focus(&mut self) {
        self.w.clear_focus();
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl<A, D> IVisible for Dialog<A, D> {}
impl<A, D> IEventHandler for Dialog<A, D> {
    type Action = A;
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action<Self::Action>> {
        trace!("Handling key event for dialog: {}", self.w.name);
        // if Escape is pressed then dismiss the dialog
        if key.code == crossterm::event::KeyCode::Esc {
            trace!("Dismissing dialog: {}", self.w.name);
            return Some(Action::new(self.w.name.clone(), UiActions::DismissDialog));
        }

        let action = self.w.handle_key_event(key);

        // if Cancel is clicked then dismiss the dialog otherwise forward action
        if let Some(action) = action {
            match action.action {
                UiActions::ButtonClicked(name) => match name.as_str() {
                    "Cancel" => {
                        return Some(Action::new(self.w.name.clone(), UiActions::DismissDialog))
                    }
                    _ => {
                        //TODO: call custom button handler to update the state
                        return None;
                    }
                },
                _ => {
                    //TODO: call custom button handler to update the state
                    return Some(action);
                }
            }
        } else {
            None
        }
    }
}
