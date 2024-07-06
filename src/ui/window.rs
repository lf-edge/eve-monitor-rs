use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
    thread::JoinHandle,
};

use log::{trace, warn};
use ratatui::{layout::Rect, Frame};

use crate::{
    dispatcher::EventDispatcher,
    events::{Event, EventCode},
    traits::VisualComponent,
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

pub struct WindowBuilder {
    views: HashMap<String, Box<dyn VisualComponent>>,
    focused_view: Option<String>,
    do_layout: Box<dyn Fn(&Rect) -> HashMap<String, Rect>>,
    name: Option<String>,
}

impl WindowBuilder {
    pub fn add_view(mut self, view: impl VisualComponent + 'static) -> Self {
        self.views.insert(view.name().to_string(), Box::new(view));
        self
    }
    pub fn with_layout(
        mut self,
        layout: impl Fn(&Rect) -> HashMap<String, Rect> + 'static,
    ) -> Self {
        self.do_layout = Box::new(layout);
        self
    }
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn build(self) -> Window {
        // if name is not set then generate a default name
        let ret = Window::new(self.name, self.views, self.focused_view, self.do_layout);
        ret
    }
}
type LayoutFn = Box<dyn Fn(&Rect) -> HashMap<String, Rect>>;

/// Window represent a root of UI hierarchy.
/// It is a container for all other UI elements.
pub struct Window {
    id: WindowId,
    views: HashMap<String, Box<dyn VisualComponent>>,
    focused_view: Option<String>,
    dispatcher: EventDispatcher<Event>,
    do_layout: LayoutFn,
    name: String,
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("id", &self.id)
            .field("views", &self.views)
            .field("focused_view", &self.focused_view)
            .finish()
    }
}

impl Window {
    pub fn builder() -> WindowBuilder {
        WindowBuilder {
            views: HashMap::new(),
            focused_view: None,
            do_layout: Box::new(|_| HashMap::new()),
            name: None,
        }
    }

    fn new(
        name: Option<String>,
        views: HashMap<String, Box<dyn VisualComponent>>,
        focused_view: Option<String>,
        do_layout: LayoutFn,
    ) -> Self {
        let id = Self::gen_window_id();
        // if name is not set then generate a default name
        let name = name.unwrap_or_else(|| format!("Window {}", id));

        Self {
            name: name.into(),
            id: id,
            views,
            focused_view: focused_view,
            dispatcher: EventDispatcher::new(),
            do_layout,
        }
    }
    pub fn gen_window_id() -> WindowId {
        WIN_ID.next()
    }
    // pub fn add_view(&mut self, view: Box<dyn VisualComponent>) {
    //     self.views.insert(view.id(), view);
    // }
    // pub fn remove_view(&mut self, id: WindowId) {
    //     self.views.remove(&id);
    // }
    // pub fn focus_view(&mut self, id: WindowId) {
    //     trace!("Focusing view {}", id);
    //     // if there is a view in focus then notify it that it is losing focus
    //     if let Some(view) = self.focused_view.and_then(|id| self.views.get_mut(&id)) {
    //         view.borrow_mut().focus_lost();
    //     }

    //     self.focused_view = Some(id);
    //     // let view know that it is in focus
    //     if let Some(view) = self.views.get_mut(&id) {
    //         view.borrow_mut().focus();
    //     } else {
    //         warn!("View with id {} not found", id)
    //     }
    // }
    // pub fn get_focused_view(&self) -> Option<&Box<dyn VisualComponent>> {
    //     self.focused_view.and_then(|id| self.views.get(&id))
    // }
    pub fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        trace!("Rendering window {} {}", self.id, self.name);
        for (_id, view) in self.views.iter_mut() {
            //get layout tfo the view
            let layout = (self.do_layout)(area);
            let area = layout.get(_id).unwrap();
            trace!("Rendering view {} at {:?}", _id, area);
            trace!("Layout: {:?}", layout);

            // if the view is in focus then render it with focus
            // let focused = &self
            //     .focused_view
            //     .and_then(|f| Some(f == *id))
            //     .unwrap_or(false);
            //VisualComponent::layout(&1mut *view, area);
            view.layout(area);
            view.render(area, frame, true);
        }
    }
    pub fn handle_event(&mut self, event: &EventCode) -> Option<EventCode> {
        trace!("window {} Event: {:?} ", self.id, event);
        trace!("focused view: {:?}", self.focused_view);
        // get children of the focused view
        // if let Some(view) = self.focused_view {
        //     let children = self
        //         .views
        //         .get_mut(&view)
        //         .and_then(|v| Some(v.get_children()))
        //         .unwrap_or(Vec::new());
        //     trace!("children: {:?}", children);
        // }

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
