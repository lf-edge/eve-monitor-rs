use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, StatefulWidgetRef, WidgetRef},
};

use crate::traits::{IFocusAcceptor, IStatefulWidgetPresenter, IVisible, IWidgetPresenter};

#[derive(Debug)]
pub struct VisualState {
    pub visible: bool,
    pub focused: bool,
}

impl Default for VisualState {
    fn default() -> Self {
        Self {
            visible: true,
            focused: false,
        }
    }
}

#[derive(Debug)]
pub struct Element<D> {
    pub v: VisualState,
    pub d: D,
}

impl<D> IVisible for Element<D>
where
    Self: IWidgetPresenter,
{
    fn is_visible(&self) -> bool {
        self.v.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.v.visible = visible;
    }
}

impl<D> IFocusAcceptor for Element<D>
where
    Self: IWidgetPresenter,
{
    fn set_focus(&mut self) {
        self.v.focused = true;
    }

    fn clear_focus(&mut self) {
        self.v.focused = false;
    }

    fn has_focus(&self) -> bool {
        self.v.focused
    }
}
// impl<D> IFocusAcceptor for Element<D>
// where
//     Self: IStatefulWidgetPresenter<State = D>,
// {
//     fn set_focus(&mut self) {
//         self.v.focused = true;
//     }

//     fn clear_focus(&mut self) {
//         self.v.focused = false;
//     }

//     fn has_focus(&self) -> bool {
//         self.v.focused
//     }
// }

impl<D> WidgetRef for Element<D>
where
    Self: IWidgetPresenter,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }
}

impl<D> WidgetRef for &mut Element<D>
where
    Self: IWidgetPresenter,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }
}

impl<D> StatefulWidgetRef for Element<D>
where
    Self: IStatefulWidgetPresenter<State = D>,
{
    type State = D;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with_state(area, buf, state);
    }
}

impl<D> StatefulWidgetRef for &mut Element<D>
where
    Self: IStatefulWidgetPresenter<State = D>,
{
    type State = D;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with_state(area, buf, state);
    }
}

impl<D> StatefulWidget for Element<D>
where
    Self: IStatefulWidgetPresenter<State = D>,
{
    type State = D;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with_state(area, buf, state);
    }
}

impl<D> StatefulWidget for &mut Element<D>
where
    Self: IStatefulWidgetPresenter<State = D>,
{
    type State = D;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with_state(area, buf, state);
    }
}

// TODO: Experimental code to wrap a widget with a name
// but let it pretend to be IWidget

// pub trait NamedElement {
//     fn name(&self) -> &str;
// }

// struct NamedElementImpl<W>
// where
//     W: IWidget,
// {
//     name: String,
//     widget: W,
// }

// impl<W> NamedElement for NamedElementImpl<W>
// where
//     W: IWidget,
// {
//     fn name(&self) -> &str {
//         &self.name
//     }
// }

// impl<W> Deref for NamedElementImpl<W>
// where
//     W: IWidget,
// {
//     type Target = W;

//     fn deref(&self) -> &Self::Target {
//         &self.widget
//     }
// }
