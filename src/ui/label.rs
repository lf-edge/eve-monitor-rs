use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{self, Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, StatefulWidgetRef, WidgetRef},
    Frame,
};

use crate::traits::VisualComponent;

use super::component::{StatefulComponentWrapper, VisualComponentState, WidgetState};

pub struct LabelWidget<'a> {
    text: Box<Paragraph<'a>>,
}

impl<'a> LabelWidget<'a> {
    pub fn new(text: String) -> Self {
        Self {
            text: Box::new(
                Paragraph::new(text)
                    .style(Style::default().fg(Color::White).bg(Color::Red))
                    .alignment(Alignment::Left),
            ),
        }
    }
}

//FIXME: we do not use this state yet
pub struct LabelWidgetState {
    _text: String,
}

impl LabelWidgetState {
    pub fn new(text: String) -> Self {
        Self { _text: text }
    }
}

impl WidgetState for LabelWidgetState {
    fn get_layout(&self) -> HashMap<String, Rect> {
        return HashMap::new();
    }
}
impl WidgetState for &LabelWidgetState {
    fn get_layout(&self) -> HashMap<String, Rect> {
        return HashMap::new();
    }
}

// impl WidgetState for VisualComponentState<LabelWidgetState> {}

impl<'a> StatefulWidgetRef for LabelWidget<'a> {
    type State = VisualComponentState<LabelWidgetState>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        self.text.render_ref(area, buf);
    }
}

// implement StatefulWidgetRef got Box<LabelWidget>
impl StatefulWidgetRef for &mut Box<LabelWidget<'_>> {
    type State = VisualComponentState<LabelWidgetState>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        self.text.render_ref(area, buf);
    }
}

pub type LabelView<'a> = StatefulComponentWrapper<LabelWidget<'a>, LabelWidgetState>;

impl<'a> LabelView<'a> {
    pub fn new<S: Into<String>>(name: S, text: S) -> Self {
        let text = text.into();
        Self::create_component_state(
            name.into(),
            Box::new(LabelWidget::new(text.clone())),
            LabelWidgetState { _text: text },
            Box::new(|_, _| HashMap::new()),
        )
    }
}

impl<'a> VisualComponent for LabelView<'a> {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
    }
}
