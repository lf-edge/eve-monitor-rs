use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
    thread::JoinHandle,
};

use log::{trace, warn};
use ratatui::{layout::Rect, widgets::StatefulWidgetRef, Frame};

use crate::{
    dispatcher::EventDispatcher,
    events::{Event, EventCode},
    traits::{Component, VisualComponent},
};

/// WindowId is a unique identifier for a window that is incremented sequentially.
pub type WindowId = usize;

struct WindowIdGenerator(AtomicUsize);
impl WindowIdGenerator {
    fn next(&self) -> WindowId {
        self.0.fetch_add(1, Ordering::SeqCst)
    }
}

// statically initialize the window id counter
static WIN_ID: WindowIdGenerator = WindowIdGenerator(AtomicUsize::new(1));
/// TARGET_APP_ID is a special identifier to roue events to the application's event loop.
pub static TARGET_APP_ID: WindowId = 0;

/// Window represent a root of UI hierarchy.
/// It is a container for all other UI elements.
#[derive(Debug)]
pub struct Window {
    id: WindowId,
    views: HashMap<WindowId, Box<dyn VisualComponent>>,
    focused_view: Option<WindowId>,
    dispatcher: EventDispatcher<Event>,
    root: WindowId,
}
impl Window {
    pub fn new(root: Box<dyn VisualComponent>) -> Self {
        let mut ret = Self {
            id: Self::gen_window_id(),
            views: HashMap::new(),
            focused_view: None,
            dispatcher: EventDispatcher::new(),
            root: root.id(),
        };
        ret.add_view(root);
        ret.focus_view(ret.root);
        ret
    }
    pub fn gen_window_id() -> WindowId {
        WIN_ID.next()
    }
    pub fn add_view(&mut self, view: Box<dyn VisualComponent>) {
        self.views.insert(view.id(), view);
    }
    pub fn remove_view(&mut self, id: WindowId) {
        self.views.remove(&id);
    }
    pub fn focus_view(&mut self, id: WindowId) {
        trace!("Focusing view {}", id);
        // if there is a view in focus then notify it that it is losing focus
        // if let Some(view) = self.focused_view.and_then(|id| self.views.get_mut(&id)) {
        //     view.focus_lost();
        // }

        self.focused_view = Some(id);
        // let view know that it is in focus
        // if let Some(view) = self.views.get_mut(&id) {
        //     view.focus_gain();
        // } else {
        //     warn!("View with id {} not found", id)
        // }
    }
    pub fn get_focused_view(&self) -> Option<&Box<dyn VisualComponent>> {
        self.focused_view.and_then(|id| self.views.get(&id))
    }
    pub fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        //trace!("Rendering window {}", self.id);
        for (id, view) in self.views.iter_mut() {
            // if the view is in focus then render it with focus
            // root view is always in focus
            let focused = (self.focused_view == Some(*id)) || (*id == self.root);
            view.render(area, frame, focused);
        }
    }
    pub fn handle_event(&mut self, event: &EventCode) -> Option<EventCode> {
        trace!("window {} Event: {:?} ", self.id, event);
        // get children of the focused view
        if let Some(view) = self.focused_view {
            let children = self
                .views
                .get_mut(&view)
                .and_then(|v| Some(v.get_children()))
                .unwrap_or(Vec::new());
            trace!("children: {:?}", children);

            // self.views
            //     .get_mut(&view)
            //     .and_then(|v| v.get_children())
            //     .and_then(|e| {
            //         for (parent, child) in e {
            //             if parent == view {
            //                 if let Some(view) = self.views.get_mut(&child) {
            //                     if let Some(e) = view.handle_event(event) {
            //                         return Some(e);
            //                     }
            //                 }
            //             }
            //         }
            //         None
            //     });
        }

        // if let Some(children) = self.views.get(&self.root).and_then(|v| v.get_children()) {
        //     for (parent, child) in children {
        //         trace!("parent: {} child: {}", parent, child);
        //     }
        // }

        // match event {
        //     EventCode::Tab => {
        //         // if the focused view returns Some(WindowId) then it could handle
        //         // focus event itself. If None is returned then the window should handle
        //         if let Some(view) = self.focused_view.and_then(|id| self.views.get_mut(&id)) {
        //             if view.can_focus() {
        //                 if let Some(next) = view.focus_next() {
        //                     self.focus_view(next);
        //                 } else {
        //                     // if the view does not handle focus event then
        //                     // the window should handle it
        //                     let mut keys: Vec<WindowId> = self.views.keys().cloned().collect();
        //                     keys.sort();
        //                     let idx = keys.iter().position(|&id| id == self.focused_view.unwrap());
        //                     if let Some(idx) = idx {
        //                         let next = if idx + 1 < keys.len() {
        //                             keys[idx + 1]
        //                         } else {
        //                             keys[0]
        //                         };
        //                         self.focus_view(next);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     _ => {}
        // }

        None
    }
}

struct WndProc {
    thread: JoinHandle<()>,
}

pub struct Dialog {}
