use ratatui::{buffer::Buffer, layout::Rect};

use crate::traits::{IFocusAcceptor, IVisible, IWidgetPresenter};

#[derive(Debug)]
pub struct VisualState {
    pub visible: bool,
    pub focused: bool,
    pub can_focus: bool,
}

impl Default for VisualState {
    fn default() -> Self {
        Self {
            visible: true,
            focused: false,
            can_focus: true,
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

    fn can_focus(&self) -> bool {
        self.v.can_focus
    }
}

pub trait IStandardRenderer {
    fn render(&self, area: &Rect, buf: &mut Buffer);
}

impl<S> IWidgetPresenter for Element<S>
where
    Self: IStandardRenderer,
{
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        // call render from IStandardRenderer
        <Element<S> as IStandardRenderer>::render(self, area, frame.buffer_mut());
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