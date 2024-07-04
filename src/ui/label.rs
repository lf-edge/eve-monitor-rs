use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
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
                    .style(Style::default().fg(Color::White))
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

impl WidgetState for LabelWidgetState {}
impl WidgetState for &LabelWidgetState {}

impl WidgetState for VisualComponentState<LabelWidgetState> {}

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

pub type Label<'a> = StatefulComponentWrapper<LabelWidget<'a>, LabelWidgetState>;

impl<'a> Label<'a> {
    pub fn new(text: String) -> Self {
        Self::create_component_state(
            Box::new(LabelWidget::new(text.clone())),
            LabelWidgetState { _text: text },
        )
    }
}

impl<'a> VisualComponent for Label<'a> {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
    }
}
