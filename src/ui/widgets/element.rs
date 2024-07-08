use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidgetRef},
    Frame,
};

use crate::traits::{IFocusAcceptor, ILayout, IPresenter, IVisible, IWidgetPresenter};

#[derive(Debug)]
pub struct WidgetWithLayout<W> {
    widget: W,
    layout: HashMap<String, Rect>,
}

impl<W> WidgetWithLayout<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            layout: HashMap::new(),
        }
    }
}

impl<W> StatefulWidgetRef for WidgetWithLayout<W>
where
    W: StatefulWidgetRef,
    W::State: ILayout,
{
    type State = W::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Now you can access ILayout methods on the state
        //state.get_layout(); //

        // Delegate to the original widget's render_ref method
        self.widget.render_ref(area, buf, state);
    }
}

impl<W> StatefulWidgetRef for &mut WidgetWithLayout<W>
where
    W: StatefulWidgetRef,
    W::State: ILayout + IVisible + IFocusAcceptor,
{
    type State = W::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Now you can access ILayout methods on the state
        //state.get_layout();

        // Delegate to the original widget's render_ref method
        self.widget.render_ref(area, buf, state);
    }
}
