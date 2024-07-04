use log::trace;
use ratatui::widgets::StatefulWidgetRef;

use crate::traits::{Component, VisualComponent};

use super::window::{Window, WindowId};

pub trait WidgetState {}

pub struct VisualComponentState<S>
where
    S: WidgetState,
{
    pub widget_state: S,
    id: WindowId,
    visible: bool,
    focused: bool,
}

impl<S> VisualComponentState<S>
where
    S: WidgetState,
{
    pub fn new(widget_state: S) -> Self {
        Self {
            id: Window::gen_window_id(),
            visible: true,
            focused: false,
            widget_state,
        }
    }
}

impl<S> VisualComponentState<S>
where
    S: WidgetState,
{
    pub fn id(&self) -> WindowId {
        self.id
    }
    pub fn focused(&self) -> bool {
        self.focused
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

// struct IdState {
//     id: WindowId,
// }

// impl IdState {
//     fn new() -> Self {
//         Self {
//             id: Window::gen_window_id(),
//         }
//     }
// }

// impl WidgetState for IdState {}

pub struct StatefulComponentWrapper<W, S>
where
    W: StatefulWidgetRef<State = VisualComponentState<S>>,
    S: WidgetState,
{
    pub widget: Box<W>,
    pub state: VisualComponentState<S>,
    pub root: Vec<Box<dyn VisualComponent>>,
}

impl<W, S> StatefulComponentWrapper<W, S>
where
    W: StatefulWidgetRef<State = VisualComponentState<S>>,
    S: WidgetState,
{
    pub fn create_component_state(widget: Box<W>, state: S) -> Self {
        Self {
            widget,
            state: VisualComponentState::new(state),
            root: Vec::new(),
        }
    }
}

impl<W, S> Component for StatefulComponentWrapper<W, S>
where
    W: StatefulWidgetRef<State = VisualComponentState<S>>,
    S: WidgetState,
{
    fn get_children(&self) -> Vec<(WindowId, WindowId)> {
        trace!(
            "Getting children of id: {}, type {}",
            self.id(),
            std::any::type_name::<W>()
        );
        // collect immediate children of the root view
        let mut children: Vec<(WindowId, WindowId)> =
            self.root.iter().map(|c| (c.id(), self.id())).collect();
        // traverse grandchildren of the root view
        for child in &self.root {
            let grandchildren = child.get_children();
            children.extend(grandchildren);
        }
        children
    }

    fn id(&self) -> WindowId {
        self.state.id()
    }
    fn focused(&self) -> bool {
        self.state.focused()
    }

    fn visible(&self) -> bool {
        self.state.visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.state.visible = visible;
    }

    fn focus(&mut self) {
        self.state.focused = true;
    }

    fn focus_lost(&mut self) {
        self.state.focused = false;
    }
}

// pub type ComponentWrapper<W> = StatefulComponentWrapper<W, IdState>;

// impl<W> ComponentWrapper<W>
// where
//     W: StatefulWidgetRef<State = IdState>,
// {
//     pub fn new_stateless_inner(widget: Box<W>) -> Self {
//         Self {
//             widget,
//             state: Box::new(IdState::new()),
//         }
//     }
// }
