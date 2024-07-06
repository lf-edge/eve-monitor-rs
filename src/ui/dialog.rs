use std::collections::HashMap;

use log::trace;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, StatefulWidgetRef, WidgetRef},
    Frame,
};

use crate::{
    events::Event,
    traits::{Component, VisualComponent},
};

use super::{
    button::{Button, OnButtonClicked},
    component::{StatefulComponentWrapper, VisualComponentState, WidgetState},
    tools::centered_rect,
    window::{Window, WindowId},
};

pub struct DialogBuilder {
    title: String,
    buttons: HashMap<String, OnButtonClicked>,
    views: HashMap<String, Box<dyn VisualComponent>>,
    do_layout: Box<dyn Fn(&DialogWidgetState, &Rect) -> HashMap<String, Rect>>,
    size: (u16, u16),
}

fn dialog_default_layout_vertical(state: &DialogWidgetState, area: &Rect) -> HashMap<String, Rect> {
    let mut result = HashMap::new();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black))
        .title(state.title.as_str());

    let inner = block.inner(*area);
    result.insert("frame".to_string(), inner);
    let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner);

    result.insert("content".to_string(), layout[0]);
    result.insert("buttons".to_string(), layout[1]);

    result
}

impl DialogBuilder {
    fn new() -> Self {
        Self {
            title: String::new(),
            buttons: HashMap::new(),
            views: HashMap::new(),
            do_layout: Box::new(dialog_default_layout_vertical),
            size: (30, 15),
        }
    }
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }
    pub fn button(mut self, label: &str, on_click: OnButtonClicked) -> Self {
        self.buttons.insert(label.to_string(), on_click);
        self
    }
    pub fn view(mut self, view: Box<dyn VisualComponent>) -> Self {
        self.views.insert(view.name().to_string(), view);
        self
    }
    pub fn with_layout(
        mut self,
        layout: impl Fn(&DialogWidgetState, &Rect) -> HashMap<String, Rect> + 'static,
    ) -> Self {
        self.do_layout = Box::new(layout);
        self
    }
    pub fn build(self) -> Window {
        let mut buttons: HashMap<String, Box<dyn VisualComponent>> = HashMap::new();
        for (label, on_click) in self.buttons {
            let button = Button::new(label.clone(), label.clone(), on_click);
            buttons.insert(label, Box::new(button));
        }
        let dlg = Dialog::new("root_view", &self.title, (30, 15), buttons, self.do_layout);
        let wnd = Window::builder()
            .add_view(dlg)
            .with_layout(move |r| {
                let mut layout = HashMap::new();
                let rect = centered_rect(self.size.0, self.size.1, r.clone());
                layout.insert("root_view".to_string(), rect);
                trace!("Dialog layout: {:?}", layout);
                layout
            })
            .name(format!("Dialog: -{}-", self.title))
            .build();
        wnd
    }
}
#[derive(Debug)]
pub struct DialogWidgetState {
    title: String,
    buttons: HashMap<String, Box<dyn VisualComponent>>,
    size: (u16, u16),
    layout_map: HashMap<String, Rect>,
}
impl WidgetState for DialogWidgetState {
    fn get_layout(&self) -> HashMap<String, Rect> {
        return self.layout_map.clone();
    }
}
pub struct DialogWidget {
    frame: Block<'static>,
}

impl DialogWidget {
    fn render(&self, state: &DialogWidgetState, area: Rect, buf: &mut Buffer) {
        trace!("Dialog render with state: {:?}", state);
        // let content_area = self.state.layout_map[&"content".to_string()];
        // let buttons_area = self.state.layout_map[&"buttons".to_string()];
        // // render the frame
        // //frame.render_stateful_widget_ref(&mut self.widget, *area);

        // // self.widget
        // //     .frame
        // //     .(self.state.widget_state.title.as_str());

        // //render buttons
        // for (i, (_label, button)) in self
        //     .state
        //     .widget_state
        //     .buttons
        //     .iter_mut()
        //     //.filter(|c| is_button(c))
        //     .enumerate()
        // {
        //     button.render(&layout_buttons[i], frame, _focused);
        // }
        // // render content
        // for c in self.root.iter_mut() {
        //     c.1.render(&content_area, frame, _focused);
        // }

        self.frame.render_ref(area, buf);

        // render content
    }
}

impl<'a> StatefulWidgetRef for Box<DialogWidget> {
    type State = VisualComponentState<DialogWidgetState>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render(&state.widget_state, area, buf);
    }
}

impl<'a> StatefulWidgetRef for &Box<DialogWidget> {
    type State = VisualComponentState<DialogWidgetState>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render(&state.widget_state, area, buf);
    }
}

impl<'a> StatefulWidgetRef for DialogWidget {
    type State = VisualComponentState<DialogWidgetState>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render(&state.widget_state, area, buf);
    }
}

pub type Dialog = StatefulComponentWrapper<DialogWidget, DialogWidgetState>;

impl Dialog {
    fn new<S: Into<String>>(
        name: S,
        title: S,
        size: (u16, u16),
        buttons: HashMap<String, Box<dyn VisualComponent>>,
        layout: Box<dyn Fn(&DialogWidgetState, &Rect) -> HashMap<String, Rect>>,
    ) -> Self {
        Self::create_component_state(
            name.into(),
            Box::new(DialogWidget {
                frame: Block::new()
                    .border_type(BorderType::Thick)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .style(Style::default().bg(Color::Black)), //.title(self.state.widget_state.title.as_str()),
            }),
            DialogWidgetState {
                title: title.into(),
                buttons,
                size,
                layout_map: HashMap::new(),
            },
            layout,
        )
    }
    pub fn builder() -> DialogBuilder {
        DialogBuilder::new()
    }
}

impl VisualComponent for Dialog {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        frame.render_stateful_widget_ref(&self.widget, *area, &mut self.state);
        let content_area = self.state.widget_state.layout_map[&"content".to_string()];
        let buttons_area = self.state.widget_state.layout_map[&"buttons".to_string()];

        let layout_buttons = Layout::horizontal(
            self.state
                .widget_state
                .buttons
                .iter()
                .map(|(label, _)| Constraint::Length(label.len() as u16 + 4))
                .collect::<Vec<_>>(),
        )
        .flex(ratatui::layout::Flex::End)
        .split(buttons_area);

        for (i, (label, button)) in self.state.widget_state.buttons.iter_mut().enumerate() {
            button.render(&layout_buttons[i], frame, false);
        }
    }

    fn handle_event(&mut self, _event: &Event) -> Option<Event> {
        None
    }

    fn layout(&mut self, area: &Rect) {
        self.state.widget_state.layout_map = (self.do_layout)(&self.state.widget_state, area);
    }
}
