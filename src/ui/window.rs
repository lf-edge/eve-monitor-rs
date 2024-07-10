// use std::{
//     borrow::BorrowMut,
//     cell::RefCell,
//     collections::HashMap,
//     fmt::Debug,
//     sync::atomic::{AtomicUsize, Ordering},
//     thread::JoinHandle,
// };

// use log::{trace, warn};
// use ratatui::{layout::Rect, Frame};

// use crate::{
//     dispatcher::EventDispatcher,
//     events::{Event, EventCode},
//     traits::VisualComponent,
// };

// #[derive(Debug)]
// pub struct FocusTracker {
//     focused_view: usize,
//     tab_order: Vec<String>,
// }

// impl FocusTracker {
//     fn new(tab_order: Vec<String>, focused_view: Option<String>) -> Self {
//         // if focused view is set find its index in the tab order
//         let focused_view = focused_view
//             .and_then(|name| tab_order.iter().position(|n| n == &name))
//             .unwrap_or(0);

//         Self {
//             focused_view,
//             tab_order,
//         }
//     }

//     pub fn create_from_taborder(
//         tab_order: Vec<String>,
//         focused_view: Option<String>,
//     ) -> FocusTracker {
//         let focus_tracker = FocusTracker::new(tab_order, focused_view);
//         focus_tracker
//     }

//     pub fn create_from_views(
//         views: &HashMap<String, Box<dyn VisualComponent>>,
//         focused_view: Option<String>,
//     ) -> FocusTracker {
//         let collect_views = || {
//             let mut tab_order = Vec::new();

//             for (view_name, view) in views.iter() {
//                 if view.can_focus() {
//                     tab_order.push(view_name.clone());
//                 }
//             }
//             tab_order
//         };

//         let tab_order = collect_views();
//         let focus_tracker = FocusTracker::new(tab_order, focused_view);
//         focus_tracker
//     }

//     pub fn get_focused_view(&self) -> Option<String> {
//         self.tab_order.get(self.focused_view).cloned()
//     }
//     pub fn focus_next(&mut self) -> Option<String> {
//         if self.focused_view + 1 < self.tab_order.len() {
//             self.focused_view += 1;
//         } else {
//             self.focused_view = 0;
//         }

//         Some(self.tab_order[self.focused_view].clone())
//     }

//     pub fn focus_prev(&mut self) -> Option<String> {
//         if self.focused_view > 0 {
//             self.focused_view -= 1;
//         } else {
//             self.focused_view = self.tab_order.len() - 1;
//         }

//         Some(self.tab_order[self.focused_view].clone())
//     }
// }

// pub struct WindowBuilder {
//     views: HashMap<String, Box<dyn VisualComponent>>,
//     focused_view: Option<String>,
//     do_layout: Box<dyn Fn(&Rect) -> HashMap<String, Rect>>,
//     name: Option<String>,
//     tab_order: Option<Vec<String>>,
// }

// impl WindowBuilder {
//     pub fn add_view(mut self, view: impl VisualComponent + 'static) -> Self {
//         let view_name = view.name().to_string();

//         self.views.insert(view_name, Box::new(view));
//         self
//     }
//     pub fn with_layout(
//         mut self,
//         layout: impl Fn(&Rect) -> HashMap<String, Rect> + 'static,
//     ) -> Self {
//         self.do_layout = Box::new(layout);
//         self
//     }
//     pub fn name<S: Into<String>>(mut self, name: S) -> Self {
//         self.name = Some(name.into());
//         self
//     }
//     pub fn focused_view<S: Into<String>>(mut self, name: S) -> Self {
//         self.focused_view = Some(name.into());
//         self
//     }
//     pub fn tab_order(mut self, order: Vec<String>) -> Self {
//         self.tab_order = Some(order);
//         self
//     }
//     pub fn build(self) -> Window {
//         let focus_tracker = if let Some(order) = self.tab_order {
//             FocusTracker::create_from_taborder(order, self.focused_view.clone())
//         } else {
//             FocusTracker::create_from_views(&self.views, self.focused_view.clone())
//         };

//         let ret = Window::new(self.name, self.views, focus_tracker, self.do_layout);
//         ret
//     }
// }
// type LayoutFn = Box<dyn Fn(&Rect) -> HashMap<String, Rect>>;

// /// Window represent a root of UI hierarchy.
// /// It is a container for all other UI elements.
// pub struct Window {
//     id: WindowId,
//     views: HashMap<String, Box<dyn VisualComponent>>,
//     dispatcher: EventDispatcher<Event>,
//     do_layout: LayoutFn,
//     name: String,
//     focus_tracker: FocusTracker,
// }

// impl Window {
//     pub fn builder() -> WindowBuilder {
//         WindowBuilder {
//             views: HashMap::new(),
//             focused_view: None,
//             do_layout: Box::new(|_| HashMap::new()),
//             name: None,
//             tab_order: None,
//         }
//     }

//     fn new(
//         name: Option<String>,
//         views: HashMap<String, Box<dyn VisualComponent>>,
//         focus_tracker: FocusTracker,
//         do_layout: LayoutFn,
//     ) -> Self {
//         let id = Self::gen_window_id();
//         // if name is not set then generate a default name
//         let name = name.unwrap_or_else(|| format!("Window {}", id));

//         Self {
//             name: name.into(),
//             id: id,
//             views,
//             focus_tracker,
//             dispatcher: EventDispatcher::new(),
//             do_layout,
//         }
//     }
//     pub fn gen_window_id() -> WindowId {
//         WIN_ID.next()
//     }

//     pub fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
//         trace!("Rendering window {} {}", self.id, self.name);
//         for (name, view) in self.views.iter_mut() {
//             //get layout for the view
//             let layout = (self.do_layout)(area);
//             let area = layout.get(name).unwrap();
//             trace!("Rendering view {} at {:?}", name, area);
//             trace!("Layout: {:?}", layout);

//             view.layout(area);
//             view.render(area, frame, true);
//         }
//     }
//     pub fn handle_event(&mut self, event: &EventCode) -> Option<EventCode> {
//         trace!("window {} Event: {:?} ", self.id, event);
//         trace!("focused view: {:#?}", self.focus_tracker);

//         let focused_view = self
//             .focus_tracker
//             .get_focused_view()
//             .and_then(|name| self.views.get_mut(&name));

//         match event {
//             EventCode::Tab => {
//                 if let Some(focused_view) = focused_view {
//                     trace!("Focused view: {}", focused_view.name());
//                     // let's try forward the event first.
//                     if !focused_view.focus_next() {
//                         // internal view lost focus
//                         // we should find the next focusable view in the current window
//                         focused_view.focus_lost();
//                         if let Some(next) = self.focus_tracker.focus_next() {
//                             let view = self.views.get_mut(&next).unwrap();
//                             view.focus();
//                             return Some(EventCode::Redraw);
//                         }
//                     } else {
//                         trace!("Focus forwarded to {}", focused_view.name());
//                     }
//                 } else {
//                     // focused view was never set
//                     trace!("No focused view found for {}", self.name);
//                     if let Some(next) = self.focus_tracker.focus_next() {
//                         trace!("Focusing next view: {}", next);
//                         let view = self.views.get_mut(&next).unwrap();
//                         view.focus();
//                         return Some(EventCode::Redraw);
//                     }
//                 }
//             }
//             EventCode::ShiftTab => {
//                 if let Some(focused_view) = focused_view {
//                     focused_view.focus_lost();
//                 }

//                 if let Some(prev) = self.focus_tracker.focus_prev() {
//                     let view = self.views.get_mut(&prev).unwrap();
//                     view.focus();
//                     return Some(EventCode::Redraw);
//                 }
//             }
//             _ => {
//                 if let Some(focused_view) = focused_view {
//                     return focused_view.handle_event(event).and_then(|a| Some(a.code));
//                 }
//             }
//         }

//         // get name of the focused view
//         // if let Some(focused_view_name) = self.focused_view.as_ref() {
//         //     // find a focusable view after the current view
//         //     let focusable_views = self
//         //         .views
//         //         .iter_mut()
//         //         .filter_map(|(name, view)| {
//         //             if view.can_focus() {
//         //                 Some(name.clone())
//         //             } else {
//         //                 None
//         //             }
//         //         })
//         //         .collect::<Vec<_>>();

//         //     trace!("focusable views: {:?}", focusable_views);

//         //     let next = focusable_views
//         //         .iter()
//         //         // .skip(|name| *name == focused_view_name)
//         //         //.filter(|name| *name != focused_view_name)
//         //         .cycle()
//         //         .next();

//         //     trace!("next: {:?}", next);

//         //     if let Some(next) = next {
//         //         self.focused_view = Some(next.clone());
//         //         let view = &mut self.views.get_mut(next).unwrap();
//         //         view.focus();
//         //         return Some(Event::redraw(None).code);
//         //     } else {
//         //         trace!("No focusable view found");
//         //     }

//         //     // if let Some((name, _)) = &next_view {
//         //     //     let view = &mut self.views.get_mut(*name).unwrap();
//         //     //     self.focused_view = Some(*name.clone());
//         //     // }
//         // } else {
//         //     // choose the first focusable view from the list of views
//         //     for (name, view) in self.views.iter_mut() {
//         //         if view.can_focus() {
//         //             self.focused_view = Some(name.clone());
//         //             view.focus();
//         //             return Some(Event::redraw(None).code);
//         //         }
//         //     }
//         // }

//         // get a view that is in focus
//         // let focused_view = self
//         //     .focused_view
//         //     .as_ref()
//         //     .and_then(|id| self.views.get_mut(id));

//         // if let Some(view) = focused_view {
//         //     // find focusable element after current
//         //     self.views
//         //         .iter()
//         //         .skip_while(|(name, _)| *name != &self.focused_view.unwrap());

//         //     // return view
//         //     //     .handle_event(&Event {
//         //     //         code: event.clone(),
//         //     //         target: None,
//         //     //     })
//         //     //     .map(|e| e.code);
//         //     return Some(Event::redraw(None).code);
//         // } else {
//         //     trace!("No view in focus selecting");
//         //     // choose the first focusable view from the list of views
//         //     for (name, view) in self.views.iter_mut() {
//         //         trace!("Checking view {} can_focus={}", name, view.can_focus());
//         //         if view.can_focus() {
//         //             self.focused_view = Some(name.clone());
//         //             view.focus();
//         //             return Some(Event::redraw(None).code);
//         //             // return view
//         //             //     .handle_event(&Event {
//         //             //         code: event.clone(),
//         //             //         target: None,
//         //             //     })
//         //             //     .map(|e| e.code);
//         //         }
//         //     }
//         // }

//         // if let Some(view) = &self
//         //     .focused_view
//         //     .as_mut()
//         //     .and_then(|id| self.views.get_mut(id))
//         // {
//         //     return view
//         //         .handle_event(&Event {
//         //             code: EventCode::Tab,
//         //             target: None,
//         //         })
//         //         .map(|e| e.code);
//         // }
//         // get children of the focused view
//         // if let Some(view) = self.focused_view {
//         //     let children = self
//         //         .views
//         //         .get_mut(&view)
//         //         .and_then(|v| Some(v.get_children()))
//         //         .unwrap_or(Vec::new());
//         //     trace!("children: {:?}", children);
//         // }

//         // if let Some(children) = self.views.get(&self.root).and_then(|v| v.get_children()) {
//         //     for (parent, child) in children {
//         //         trace!("parent: {} child: {}", parent, child);
//         //     }
//         // }

//         // match event {
//         //     EventCode::Tab => {
//         //         // if the focused view returns Some(WindowId) then it could handle
//         //         // focus event itself. If None is returned then the window should handle
//         //         if let Some(view) = self.focused_view.and_then(|id| self.views.get_mut(&id)) {
//         //             if view.can_focus() {
//         //                 if let Some(next) = view.focus_next() {
//         //                     self.focus_view(next);
//         //                 } else {
//         //                     // if the view does not handle focus event then
//         //                     // the window should handle it
//         //                     let mut keys: Vec<WindowId> = self.views.keys().cloned().collect();
//         //                     keys.sort();
//         //                     let idx = keys.iter().position(|&id| id == self.focused_view.unwrap());
//         //                     if let Some(idx) = idx {
//         //                         let next = if idx + 1 < keys.len() {
//         //                             keys[idx + 1]
//         //                         } else {
//         //                             keys[0]
//         //                         };
//         //                         self.focus_view(next);
//         //                     }
//         //                 }
//         //             }
//         //         }
//         //     }
//         //     _ => {}
//         // }

//         None
//     }
// }

// struct WndProc {
//     thread: JoinHandle<()>,
// }

use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crossterm::event::{KeyCode, KeyEvent};
use log::{info, trace, warn};
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    dispatcher::EventDispatcher,
    events::{Event, UiCommand},
    traits::{
        IEventDispatcher, IEventHandler, IFocusAcceptor, IFocusTracker, IPresenter, IVisible,
        IVisibleElement, IWidget, IWindow,
    },
};
use anyhow::anyhow;
use anyhow::Result;

use super::{
    focus_tracker::{FocusMode, FocusTracker},
    tools::ElementHashMap,
};

pub type WidgetMap = ElementHashMap<Box<dyn IWidget>>;
pub type LayoutMap = ElementHashMap<Rect>;

pub type LayoutFn = Box<dyn FnMut(&Rect, &mut LayoutMap) -> Result<()>>;
pub type RenderFn = Box<dyn FnMut(&Rect, &mut ratatui::Frame<'_>, &LayoutMap, &mut WidgetMap)>;

pub struct WindowBuilder {
    name: String,
    widgets: WidgetMap,
    // callback for layout
    do_layout: Option<LayoutFn>,
    // callback for rendering
    do_render: Option<RenderFn>,
    // taborder
    tab_order: Option<Vec<String>>,
    // initial focus
    focused_view: Option<String>,
}

impl WindowBuilder {
    pub fn widget<S: Into<String>>(mut self, name: S, widget: Box<dyn IWidget>) -> Self {
        self.widgets
            .add(name.into(), widget)
            .expect("Widget name already exists");
        self
    }

    pub fn with_layout(
        mut self,
        do_layout: Box<dyn FnMut(&Rect, &mut LayoutMap) -> Result<()>>,
    ) -> Self {
        self.do_layout = Some(do_layout);
        self
    }

    pub fn with_render(mut self, do_render: RenderFn) -> Self {
        self.do_render = Some(do_render);
        self
    }

    pub fn with_taborder(mut self, tab_order: Vec<String>) -> Self {
        self.tab_order = Some(tab_order);
        self
    }

    pub fn with_focused_view<S: Into<String>>(mut self, name: S) -> Self {
        self.focused_view = Some(name.into());
        self
    }

    pub fn build(self) -> Result<Window> {
        let do_layout = self
            .do_layout
            .ok_or_else(|| anyhow!("Layout function should be set for {}", self.name))?;
        let do_render = self
            .do_render
            .ok_or_else(|| anyhow!("Render function should be set for {}", self.name))?;

        //TODO: check focused view exists in widgets
        let ft = if let Some(order) = self.tab_order {
            FocusTracker::create_from_taborder(order, self.focused_view, FocusMode::Wrap)
        } else {
            FocusTracker::create_from_views(&self.widgets, self.focused_view, FocusMode::Wrap)
        };

        Ok(Window::new(
            &self.name,
            ft,
            self.widgets,
            do_layout,
            do_render,
        ))
    }
}

pub struct Window {
    pub name: String,
    pub ft: FocusTracker,
    pub widgets: ElementHashMap<Box<dyn IWidget>>,
    pub layout: ElementHashMap<Rect>,
    pub do_layout: LayoutFn,
    pub do_render: RenderFn,
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Window {
    pub(self) fn new<S: Into<String>>(
        name: S,
        ft: FocusTracker,
        widgets: WidgetMap,
        do_layout: LayoutFn,
        do_render: RenderFn,
    ) -> Self {
        Self {
            name: name.into(),
            ft,
            widgets,
            layout: ElementHashMap::new(),
            do_layout,
            do_render,
        }
    }

    pub fn builder<S: Into<String>>(name: S) -> WindowBuilder {
        WindowBuilder {
            name: name.into(),
            widgets: ElementHashMap::new(),
            do_layout: None,
            do_render: None,
            tab_order: None,
            focused_view: None,
        }
    }
}

impl IWindow for Window {}
impl IEventDispatcher for Window {
    fn dispatch_event(&self, _event: UiCommand) {
        todo!()
    }
}
impl IEventHandler for Window {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Event> {
        // forward the event to the focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(&focused_view).unwrap();
            if let Some(evet) = widget.handle_key_event(key) {
                match evet {
                    Event::UiCommand(cmd) => {
                        //self.dispatch_event(cmd);
                        return Some(Event::UiCommand(cmd));
                    }
                    Event::Key(_) => {}
                }
            }
        }
        None
    }
}

impl IFocusTracker for Window {
    fn focus_next(&mut self) -> Option<String> {
        info!("focus_next: MainWnd {:#?}", &self.ft);

        // Clear focus from the current focused view, if there is one
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.clear_focus();
            } else {
                warn!("Focused view not found in widgets: {}", focused_view);
            }
        }

        // Loop to find the next view that can take focus
        loop {
            let next = self.ft.focus_next();
            trace!("Next focused view candidate: {:#?}", &next);

            match next {
                Some(focused_view) => {
                    if let Some(widget) = self.widgets.get_mut(&focused_view) {
                        if widget.is_focus_tracker() {
                            widget.set_focus();
                            return Some(focused_view);
                        } else {
                            trace!("Next focused view is not a focus tracker: {}", focused_view);
                        }
                    } else {
                        warn!("Next focused view not found in widgets: {}", focused_view);
                    }
                }
                None => {
                    // Break the loop if there are no more views to focus on
                    return None;
                }
            }
        }
    }

    fn focus_prev(&mut self) -> Option<String> {
        info!("focus_prev: MainWnd {:#?}", &self.ft);
        // Clear focus from the current focused view, if there is one
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.clear_focus();
            } else {
                warn!("Focused view not found in widgets: {}", focused_view);
            }
        }

        // Loop to find the next view that can take focus
        loop {
            let next = self.ft.focus_prev();
            trace!("Next focused view candidate: {:#?}", &next);

            match next {
                Some(focused_view) => {
                    if let Some(widget) = self.widgets.get_mut(&focused_view) {
                        if widget.is_focus_tracker() {
                            widget.set_focus();
                            return Some(focused_view);
                        } else {
                            trace!("Next focused view is not a focus tracker: {}", focused_view);
                        }
                    } else {
                        warn!("Next focused view not found in widgets: {}", focused_view);
                    }
                }
                None => {
                    // Break the loop if there are no more views to focus on
                    return None;
                }
            }
        }
    }

    fn get_focused_view_name(&self) -> Option<String> {
        self.ft.get_focused_view()
    }
}

impl IVisible for Window {}
impl IFocusAcceptor for Window {}
impl IPresenter for Window {
    fn do_layout(
        &mut self,
        area: &Rect,
    ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
        (self.do_layout)(area, &mut self.layout).unwrap();
        //TODO: do we need upper layer to know about the layout? probably not
        HashMap::new()
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        // set focus
        // TODO: IMO it should't be here
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.set_focus();
            }
        }
        (self.do_render)(area, frame, &self.layout, &mut self.widgets);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
