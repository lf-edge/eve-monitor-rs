// use std::collections::HashMap;

// use ratatui::{
//     buffer::Buffer,
//     layout::Rect,
//     widgets::{Paragraph, StatefulWidgetRef},
//     Frame,
// };

// use crate::{
//     traits::{IFocusAcceptor, IFocusTracker, ILayout, IPresenter, IVisible, IWidgetPresenter},
//     ui::focus_tracker::{FocusMode, FocusTracker},
// };

// #[derive(Debug)]
// pub struct VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     widget: W,
//     state: S,
//     layout: HashMap<String, Rect>,
//     pub in_focus: bool,
//     pub is_visible: bool,
//     ft: FocusTracker,
// }

// impl<W, S> VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     // pub fn new(state: S) -> Self {
//     //     Self {
//     //         widget: W::new(),
//     //         state,
//     //         layout: HashMap::new(),
//     //         in_focus: false,
//     //         is_visible: true,
//     //         ft: FocusTracker::new(Vec::new(), None, FocusMode::Wrap),
//     //     }
//     // }
// }

// impl<W, S> StatefulWidgetRef for VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     type State = S;

//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         // Now you can access ILayout methods on the state
//         //state.get_layout(); //

//         // Delegate to the original widget's render_ref method
//         self.widget.render(area, buf);
//     }
// }

// impl<W, S> StatefulWidgetRef for &mut VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     type State = S;

//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         // Now you can access ILayout methods on the state
//         //state.get_layout(); //

//         // Delegate to the original widget's render_ref method
//         self.widget.render(area, buf);
//     }
// }

// // impl<W, S> IVisible for &mut VisibleElement<W, S> {
// //     fn is_visible(&self) -> bool {
// //         self.is_visible
// //     }

// //     fn set_visible(&mut self, visible: bool) {
// //         self.is_visible = visible;
// //     }
// // }

// // impl<W, S> IFocusAcceptor for &mut VisibleElement<W, S> {
// //     fn set_focus(&mut self) {
// //         self.in_focus = true;
// //     }

// //     fn clear_focus(&mut self) {
// //         self.in_focus = false;
// //     }
// // }

// impl<W, S> IVisible for VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     fn is_visible(&self) -> bool {
//         self.is_visible
//     }

//     fn set_visible(&mut self, visible: bool) {
//         self.is_visible = visible;
//     }
// }

// impl<W, S> IFocusAcceptor for VisibleElement<W, S>
// where
//     W: IWidgetPresenter,
// {
//     fn set_focus(&mut self) {
//         self.in_focus = true;
//     }

//     fn clear_focus(&mut self) {
//         self.in_focus = false;
//     }
// }

// // impl<W> IFocusTracker for VisibleElement<W>
// // where
// //     VisibleElement<W>: ILayout + IVisible + IFocusAcceptor + IFocusTracker,
// //     W: StatefulWidgetRef,
// // {
// //     fn focus_next(&mut self) -> Option<&String> {
// //         self.ft.focus_next()
// //     }

// //     fn focus_prev(&mut self) -> Option<&String> {
// //         self.ft.focus_prev()
// //     }

// //     fn get_focused_view_name(&self) -> Option<&String> {
// //         self.ft.get_focused_view()
// //     }
// // }

use ratatui::widgets::{StatefulWidgetRef, WidgetRef};

use crate::traits::{IFocusAcceptor, IVisible, IWidgetPresenter};

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
}

impl<D> StatefulWidgetRef for Element<D>
where
    Self: IWidgetPresenter,
{
    type State = D;

    fn render_ref(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        self.render(area, buf);
    }
}

impl<D> WidgetRef for Element<D>
where
    Self: IWidgetPresenter,
{
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.render(area, buf);
    }
}

impl<D> WidgetRef for &mut Element<D>
where
    Self: IWidgetPresenter,
{
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.render(area, buf);
    }
}

impl<D> StatefulWidgetRef for &mut Element<D>
where
    Self: IWidgetPresenter,
{
    type State = D;

    fn render_ref(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        self.render(area, buf);
    }
}
