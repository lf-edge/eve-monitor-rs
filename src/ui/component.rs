use log::trace;
use ratatui::{layout::Rect, widgets::StatefulWidgetRef, Frame};

use crate::traits::{Component, VisualComponent};

use super::window::{Window, WindowId};

pub trait WidgetState {}

pub struct VisualComponentState {
    id: WindowId,
    visible: bool,
    focused: bool,
}

impl VisualComponentState {
    pub fn new() -> Self {
        Self {
            id: Window::gen_window_id(),
            visible: true,
            focused: false,
        }
    }
}

impl VisualComponentState {
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
    W: StatefulWidgetRef<State = S>,
    S: WidgetState,
{
    pub widget: Box<W>,
    pub state: Box<S>,
    pub root: Vec<Box<dyn VisualComponent>>,
    pub visual_state: VisualComponentState,
}

impl<W, S> StatefulComponentWrapper<W, S>
where
    W: StatefulWidgetRef<State = S>,
    S: WidgetState,
{
    pub fn create_component_state(widget: Box<W>, state: Box<S>) -> Self {
        Self {
            widget,
            state,
            root: Vec::new(),
            visual_state: VisualComponentState::new(),
        }
    }
}

impl<W, S> Component for StatefulComponentWrapper<W, S>
where
    W: StatefulWidgetRef<State = S>,
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
        // traverse children of the root view
        // let mut ret: Vec<(usize, usize)> = children.clone();
        for child in &self.root {
            let grandchildren = child.get_children();
            children.extend(grandchildren);
        }
        children
    }

    fn id(&self) -> WindowId {
        self.visual_state.id()
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
