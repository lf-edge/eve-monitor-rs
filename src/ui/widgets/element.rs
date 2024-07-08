use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidgetRef},
    Frame,
};

use crate::traits::{IFocusAcceptor, IPresenter, IVisible, IWidgetPresenter};

pub struct Element<S, W>
// where
//     W: IWidgetPresenter,
{
    /// widget state contains all the data displayed by widget
    pub widget_state: S,
    /// widget is the actual widget that will be rendered
    pub widget: W,
    pub visible: bool,
    pub focused: bool,
    pub name: String,
}

pub struct LabelWidgetState {
    /// text displayed by the label
    pub text: String,
}

pub struct LabelWidget<'a> {
    /// text widget that will be rendered
    pub text_widget: Box<Paragraph<'a>>,
}
// impl IWidgetPresenter for LabelWidget<'_> {
//     fn render(&self, area: Rect, buf: &mut Buffer) {
//         self.
//     }
// }

impl<'a> StatefulWidgetRef for Element<LabelWidgetState, LabelWidget<'a>>
where
    Element<LabelWidgetState, LabelWidget<'a>>: IPresenter,
{
    type State = LabelWidgetState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        //self.widget.render_ref(area, buf);
    }
}

impl<'a> IPresenter for Element<LabelWidgetState, LabelWidget<'a>> {
    fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect> {
        todo!()
    }

    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        todo!()
    }
}

impl<'a> IVisible for Element<LabelWidgetState, LabelWidget<'a>> {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl<'a> IFocusAcceptor for Element<LabelWidgetState, LabelWidget<'a>> {
    fn set_focus(&mut self) {
        self.focused = true;
    }

    fn clear_focus(&mut self) {
        self.focused = false;
    }
}

pub trait IWidget: IPresenter + StatefulWidgetRef {}
