use crate::events;
use crate::traits::IElementEventHandler;
use crate::ui::activity::Activity;
use log::debug;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

use crossterm::event::KeyEvent;
use log::{info, trace};
use ratatui::{
    layout::{self, Constraint, Flex, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
    Frame,
};

use crate::traits::{IEventHandler, IPresenter, IVisible, IWindow};

use super::{
    action::{Action, UiActions},
    focus_tracker::FocusTracker,
    tools::centered_rect_fixed,
    widgets::button::ButtonElement,
    window::{LayoutMap, WidgetMap},
};

pub struct Dialog<D>
// where
// D: IPresenter,
{
    name: String,
    focus: FocusTracker,
    size: (u16, u16),
    buttons: Vec<String>,
    state: D,
    layout: LayoutMap,
    widgets: WidgetMap,
}

impl<D: 'static> Dialog<D> {
    pub fn new(
        size: (u16, u16),
        name: String,
        buttons: Vec<String>,
        focused_button: &str,
        state: D,
    ) -> Self {
        // create buttons and add them to the window builder
        let mut widgets = WidgetMap::new();
        for button_name in buttons.iter() {
            let button = ButtonElement::new(button_name);
            widgets
                .add(button_name.to_string(), Box::new(button))
                .expect("Widget name already exists");
        }

        let focus = FocusTracker::new(
            buttons.clone(),
            Some(focused_button.to_string()),
            super::focus_tracker::FocusMode::Wrap,
        );

        Self {
            name,
            focus,
            widgets,
            size,
            buttons,
            state,
            layout: LayoutMap::new(),
        }
    }

    fn on_ok_yes<F>(_f: F) -> Option<UiActions>
    where
        F: Fn(&D) -> Option<UiActions>,
    {
        Some(UiActions::ButtonClicked("Ok".to_string()))
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

    fn render_contents(&self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        info!("Rendering dialog content");
        frame.render_widget(Paragraph::new("AAAAAARRRRRRGGGGHHHHH"), *area);
    }
}

impl<D: 'static> IWindow for Dialog<D> {}

impl<D: 'static> IPresenter for Dialog<D> {
    // fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect> {
    //     self.do_layout(area);
    //     // get content area and pass it to window
    //     let content_area = self.layout.get("content").unwrap();

    //     self.w.do_layout(&content_area);
    //     HashMap::new()
    // }

    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, dialog_focused: bool) {
        trace!("Rendering dialog: {}", self.name);
        self.do_layout(area);

        // render the dialog
        let frame_rect = self.layout.get("frame").unwrap();
        Clear.render(*frame_rect, frame.buffer_mut());

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.name.as_str());

        block.render(*frame_rect, frame.buffer_mut());

        let focused_button = self
            .focus
            .get_focused_view()
            .or(Some("".to_string()))
            .unwrap();

        debug!("focused button: {focused_button}");

        // render the buttons
        for button_name in self.buttons.iter() {
            let button_rect = self.layout.get(button_name).unwrap();
            let button = self.widgets.get_mut(button_name).unwrap();
            button.render(
                button_rect,
                frame,
                (*button_name == focused_button) && dialog_focused,
            );
        }

        // render the content
        // if let Some(self.state){}
        let content_area = self.layout.get("content").unwrap().clone();
        self.render_contents(&content_area, frame, dialog_focused);
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl<D> IVisible for Dialog<D> {}
impl<D> IEventHandler for Dialog<D> {
    fn handle_event(&mut self, event: events::Event) -> Option<Action> {
        match event {
            events::Event::Key(key) => {
                if let Some(act) = self.handle_key_event(key) {
                    match act {
                        Activity::Action(action) => {
                            return Some(Action::new(self.name.clone(), action))
                        }
                        Activity::Event(_) => (),
                    }
                }
                None
            }
            _ => None,
        }
    }
}
impl<D> IElementEventHandler for Dialog<D> {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Activity> {
        trace!("Handling key event for dialog {}: {:?}", self.name, key);
        // if Escape is pressed then dismiss the dialog
        if key.code == crossterm::event::KeyCode::Esc {
            trace!("Dismissing dialog: {}", self.name);
            return Some(Activity::Action(UiActions::DismissDialog));
        }

        if let Some(act) = self.focus.handle_key_event(key) {
            match act {
                Activity::Action(action) => {
                    if let UiActions::ButtonClicked(ref name) = action {
                        if name == "Cancel" {
                            return Some(Activity::Action(UiActions::DismissDialog));
                        }
                    }

                    return Some(Activity::Action(action));
                }
                Activity::Event(key) => {
                    if let Some(elem_name) = self.focus.get_focused_view() {
                        self.widgets
                            .get_mut(&elem_name)
                            .unwrap()
                            .handle_key_event(key)
                    } else {
                        None
                    }
                }
            }
        } else {
            return None;
        }
    }
}
