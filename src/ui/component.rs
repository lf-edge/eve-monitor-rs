// use std::collections::HashMap;

// use log::trace;
// use ratatui::{
//     layout::{self, Rect},
//     widgets::StatefulWidgetRef,
// };

// use crate::traits::{Component, VisualComponent};

// use super::window::{FocusTracker, Window, WindowId};

// pub trait WidgetState {
//     fn get_layout(&self) -> HashMap<String, Rect>;
// }

// #[derive(Debug)]
// pub struct VisualComponentState<S>
// where
//     S: WidgetState,
// {
//     pub widget_state: S,
//     id: WindowId,
//     visible: bool,
//     focused: bool,
//     pub focus_tracker: Option<FocusTracker>,
//     // pub layout_map: HashMap<String, Rect>,
// }

// impl<S> VisualComponentState<S>
// where
//     S: WidgetState,
// {
//     pub fn new(widget_state: S, focus_tracker: Option<FocusTracker>) -> Self {
//         Self {
//             id: Window::gen_window_id(),
//             visible: true,
//             focused: false,
//             widget_state,
//             focus_tracker,
//             // layout_map: HashMap::new(),
//         }
//     }
// }

// impl<S> VisualComponentState<S>
// where
//     S: WidgetState,
// {
//     pub fn id(&self) -> WindowId {
//         self.id
//     }
//     pub fn focused(&self) -> bool {
//         self.focused
//     }
//     pub fn visible(&self) -> bool {
//         self.visible
//     }
// }

// // struct IdState {
// //     id: WindowId,
// // }

// // impl IdState {
// //     fn new() -> Self {
// //         Self {
// //             id: Window::gen_window_id(),
// //         }
// //     }
// // }

// // impl WidgetState for IdState {}

// pub struct StatefulComponent<W, S>
// where
//     W: StatefulWidgetRef<State = VisualComponentState<S>>,
//     S: WidgetState,
// {
//     pub name: String,
//     pub widget: Box<W>,
//     pub state: VisualComponentState<S>,
//     pub root: HashMap<String, Box<dyn VisualComponent>>,
//     pub do_layout: Box<dyn Fn(&S, &Rect) -> HashMap<String, Rect>>,
// }

// impl<W, S> StatefulComponent<W, S>
// where
//     W: StatefulWidgetRef<State = VisualComponentState<S>>,
//     S: WidgetState,
// {
//     pub fn create_component_state<N: Into<String>>(
//         name: N,
//         widget: Box<W>,
//         state: S,
//         layout: Box<dyn Fn(&S, &Rect) -> HashMap<String, Rect>>,
//         focus_tracker: Option<FocusTracker>,
//     ) -> Self {
//         Self {
//             widget,
//             state: VisualComponentState::new(state, focus_tracker),
//             name: name.into(),
//             root: HashMap::new(),
//             do_layout: layout,
//         }
//     }
//     // pub fn layout(&mut self, area: &Rect) {
//     //     self.state.layout = (self.do_layout)(area);
//     // }
// }

// impl<W, S> Component for StatefulComponent<W, S>
// where
//     W: StatefulWidgetRef<State = VisualComponentState<S>>,
//     S: WidgetState,
// {
//     // fn get_children(&self) -> Vec<(WindowId, WindowId)> {
//     //     trace!(
//     //         "Getting children of id: {}, type {}",
//     //         self.id(),
//     //         std::any::type_name::<W>()
//     //     );
//     //     // collect immediate children of the root view
//     //     let mut children: Vec<(WindowId, WindowId)> =
//     //         self.root.iter().map(|c| (c.id(), self.id())).collect();
//     //     // traverse grandchildren of the root view
//     //     for child in &self.root {
//     //         let grandchildren = child.get_children();
//     //         children.extend(grandchildren);
//     //     }
//     //     children
//     // }

//     fn id(&self) -> WindowId {
//         self.state.id()
//     }

//     fn visible(&self) -> bool {
//         self.state.visible()
//     }

//     fn set_visible(&mut self, visible: bool) {
//         self.state.visible = visible;
//     }

//     fn focus(&mut self) {
//         trace!("Focus for {}", self.name.as_str());
//         self.state.focused = true;
//     }

//     fn focus_lost(&mut self) {
//         self.state.focused = false;
//     }

//     fn name(&self) -> &str {
//         self.name.as_str()
//     }

//     fn focus_next(&mut self) -> bool {
//         trace!("Focus next for {}", self.name.as_str());
//         // get focus tracker
//         let focused_view = self
//             .state
//             .focus_tracker
//             .as_ref()
//             .and_then(|f| f.get_focused_view());
//         let next_view = self
//             .state
//             .focus_tracker
//             .as_mut()
//             .and_then(|f| f.focus_next());

//         focused_view.map(|v| self.get_view_mut(&v).map(|c| c.focus_lost()));
//         next_view
//             .map(|v| {
//                 self.get_view_mut(&v)
//                     .map(|c| {
//                         c.focus();
//                         Some(true)
//                     })
//                     .flatten()
//                     .unwrap_or(false)
//             })
//             .unwrap()
//         // let ret = next_view.map(|v| {
//         //     self.get_view_mut(&v)
//         //         .and_then(|c| {
//         //             c.focus();
//         //             Some(true)
//         //         })
//         //         .unwrap_or(Some(false))
//         // });

//         // if let Some(focus_tracker) = &mut self.state.focus_tracker {
//         //     trace!("Focus tracker: {:#?}", focus_tracker);
//         //     // loose focus on the current window if any
//         //     if let Some(current_view) = focus_tracker.get_focused_view() {
//         //         self.get_view_mut(&current_view).map(|c| c.focus_lost());
//         //     }
//         //     // focus_tracker
//         //     //     .get_focused_view()
//         //     //     .and_then(|v| self.root.get_mut(&v))
//         //     //     .map(|c| c.focus_lost());

//         //     // get the next window id
//         //     if let Some(next_id) = focus_tracker.focus_next() {
//         //         trace!("Next id: {}", next_id);
//         //         // focus the next window
//         //         self.get_view_mut(&next_id).map(|c| c.focus());
//         //         return true;
//         //     }
//         // }
//         // false
//     }

//     fn get_view_mut(&mut self, name: &str) -> Option<&mut Box<dyn VisualComponent>> {
//         self.root.get_mut(name)
//     }
// }

// pub type StatelessComponent<W> = StatefulComponent<W, ()>;

// impl<W> StatelessComponent<W>
// where
//     W: WidgetRef,
// {
//     pub fn new_stateless_inner(widget: Box<W>) -> Self {
//         Self {
//             widget,
//             state: Box::new(()),
//             name: String::new(),
//             root: HashMap::new(),
//             do_layout: Box::new(|_, _| HashMap::new()),
//         }
//     }
// }
