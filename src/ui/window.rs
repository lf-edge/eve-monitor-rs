use std::{
    borrow::BorrowMut,
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

#[derive(Debug)]
pub struct FocusTracker {
    focused_view: usize,
    tab_order: Vec<String>,
}

impl FocusTracker {
    fn new(tab_order: Vec<String>, focused_view: Option<String>) -> Self {
        // if focused view is set find its index in the tab order
        let focused_view = focused_view
            .and_then(|name| tab_order.iter().position(|n| n == &name))
            .unwrap_or(0);

        Self {
            focused_view,
            tab_order,
        }
    }

    pub fn create_from_taborder(
        tab_order: Vec<String>,
        focused_view: Option<String>,
    ) -> FocusTracker {
        let focus_tracker = FocusTracker::new(tab_order, focused_view);
        focus_tracker
    }

    pub fn create_from_views(
        views: &HashMap<String, Box<dyn VisualComponent>>,
        focused_view: Option<String>,
    ) -> FocusTracker {
        let collect_views = || {
            let mut tab_order = Vec::new();

            for (view_name, view) in views.iter() {
                if view.can_focus() {
                    tab_order.push(view_name.clone());
                }
            }
            tab_order
        };

        let tab_order = collect_views();
        let focus_tracker = FocusTracker::new(tab_order, focused_view);
        focus_tracker
    }

    pub fn get_focused_view(&self) -> Option<String> {
        self.tab_order.get(self.focused_view).cloned()
    }
    pub fn focus_next(&mut self) -> Option<String> {
        if self.focused_view + 1 < self.tab_order.len() {
            self.focused_view += 1;
        } else {
            self.focused_view = 0;
        }

        Some(self.tab_order[self.focused_view].clone())
    }

    pub fn focus_prev(&mut self) -> Option<String> {
        if self.focused_view > 0 {
            self.focused_view -= 1;
        } else {
            self.focused_view = self.tab_order.len() - 1;
        }

        Some(self.tab_order[self.focused_view].clone())
    }
}

pub struct WindowBuilder {
    views: HashMap<String, Box<dyn VisualComponent>>,
    focused_view: Option<String>,
    do_layout: Box<dyn Fn(&Rect) -> HashMap<String, Rect>>,
    name: Option<String>,
    tab_order: Option<Vec<String>>,
}

impl WindowBuilder {
    pub fn add_view(mut self, view: impl VisualComponent + 'static) -> Self {
        let view_name = view.name().to_string();

        self.views.insert(view_name, Box::new(view));
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
    pub fn focused_view<S: Into<String>>(mut self, name: S) -> Self {
        self.focused_view = Some(name.into());
        self
    }
    pub fn tab_order(mut self, order: Vec<String>) -> Self {
        self.tab_order = Some(order);
        self
    }
    pub fn build(self) -> Window {
        let focus_tracker = if let Some(order) = self.tab_order {
            FocusTracker::create_from_taborder(order, self.focused_view.clone())
        } else {
            FocusTracker::create_from_views(&self.views, self.focused_view.clone())
        };

        let ret = Window::new(self.name, self.views, focus_tracker, self.do_layout);
        ret
    }
}
type LayoutFn = Box<dyn Fn(&Rect) -> HashMap<String, Rect>>;

/// Window represent a root of UI hierarchy.
/// It is a container for all other UI elements.
pub struct Window {
    id: WindowId,
    views: HashMap<String, Box<dyn VisualComponent>>,
    dispatcher: EventDispatcher<Event>,
    do_layout: LayoutFn,
    name: String,
    focus_tracker: FocusTracker,
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("id", &self.id)
            .field("views", &self.views)
            .field("focus_tracker", &self.focus_tracker)
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
            tab_order: None,
        }
    }

    fn new(
        name: Option<String>,
        views: HashMap<String, Box<dyn VisualComponent>>,
        focus_tracker: FocusTracker,
        do_layout: LayoutFn,
    ) -> Self {
        let id = Self::gen_window_id();
        // if name is not set then generate a default name
        let name = name.unwrap_or_else(|| format!("Window {}", id));

        Self {
            name: name.into(),
            id: id,
            views,
            focus_tracker,
            dispatcher: EventDispatcher::new(),
            do_layout,
        }
    }
    pub fn gen_window_id() -> WindowId {
        WIN_ID.next()
    }

    pub fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        trace!("Rendering window {} {}", self.id, self.name);
        for (name, view) in self.views.iter_mut() {
            //get layout for the view
            let layout = (self.do_layout)(area);
            let area = layout.get(name).unwrap();
            trace!("Rendering view {} at {:?}", name, area);
            trace!("Layout: {:?}", layout);

            view.layout(area);
            view.render(area, frame, true);
        }
    }
    pub fn handle_event(&mut self, event: &EventCode) -> Option<EventCode> {
        trace!("window {} Event: {:?} ", self.id, event);
        trace!("focused view: {:#?}", self.focus_tracker);

        let focused_view = self
            .focus_tracker
            .get_focused_view()
            .and_then(|name| self.views.get_mut(&name));

        match event {
            EventCode::Tab => {
                if let Some(focused_view) = focused_view {
                    trace!("Focused view: {}", focused_view.name());
                    // let's try forward the event first.
                    if !focused_view.focus_next() {
                        // internal view lost focus
                        // we should find the next focusable view in the current window
                        focused_view.focus_lost();
                        if let Some(next) = self.focus_tracker.focus_next() {
                            let view = self.views.get_mut(&next).unwrap();
                            view.focus();
                            return Some(EventCode::Redraw);
                        }
                    } else {
                        trace!("Focus forwarded to {}", focused_view.name());
                    }
                } else {
                    // focused view was never set
                    trace!("No focused view found for {}", self.name);
                    if let Some(next) = self.focus_tracker.focus_next() {
                        trace!("Focusing next view: {}", next);
                        let view = self.views.get_mut(&next).unwrap();
                        view.focus();
                        return Some(EventCode::Redraw);
                    }
                }
            }
            EventCode::ShiftTab => {
                if let Some(focused_view) = focused_view {
                    focused_view.focus_lost();
                }

                if let Some(prev) = self.focus_tracker.focus_prev() {
                    let view = self.views.get_mut(&prev).unwrap();
                    view.focus();
                    return Some(EventCode::Redraw);
                }
            }
            _ => {
                if let Some(focused_view) = focused_view {
                    return focused_view.handle_event(event).and_then(|a| Some(a.code));
                }
            }
        }

        // get name of the focused view
        // if let Some(focused_view_name) = self.focused_view.as_ref() {
        //     // find a focusable view after the current view
        //     let focusable_views = self
        //         .views
        //         .iter_mut()
        //         .filter_map(|(name, view)| {
        //             if view.can_focus() {
        //                 Some(name.clone())
        //             } else {
        //                 None
        //             }
        //         })
        //         .collect::<Vec<_>>();

        //     trace!("focusable views: {:?}", focusable_views);

        //     let next = focusable_views
        //         .iter()
        //         // .skip(|name| *name == focused_view_name)
        //         //.filter(|name| *name != focused_view_name)
        //         .cycle()
        //         .next();

        //     trace!("next: {:?}", next);

        //     if let Some(next) = next {
        //         self.focused_view = Some(next.clone());
        //         let view = &mut self.views.get_mut(next).unwrap();
        //         view.focus();
        //         return Some(Event::redraw(None).code);
        //     } else {
        //         trace!("No focusable view found");
        //     }

        //     // if let Some((name, _)) = &next_view {
        //     //     let view = &mut self.views.get_mut(*name).unwrap();
        //     //     self.focused_view = Some(*name.clone());
        //     // }
        // } else {
        //     // choose the first focusable view from the list of views
        //     for (name, view) in self.views.iter_mut() {
        //         if view.can_focus() {
        //             self.focused_view = Some(name.clone());
        //             view.focus();
        //             return Some(Event::redraw(None).code);
        //         }
        //     }
        // }

        // get a view that is in focus
        // let focused_view = self
        //     .focused_view
        //     .as_ref()
        //     .and_then(|id| self.views.get_mut(id));

        // if let Some(view) = focused_view {
        //     // find focusable element after current
        //     self.views
        //         .iter()
        //         .skip_while(|(name, _)| *name != &self.focused_view.unwrap());

        //     // return view
        //     //     .handle_event(&Event {
        //     //         code: event.clone(),
        //     //         target: None,
        //     //     })
        //     //     .map(|e| e.code);
        //     return Some(Event::redraw(None).code);
        // } else {
        //     trace!("No view in focus selecting");
        //     // choose the first focusable view from the list of views
        //     for (name, view) in self.views.iter_mut() {
        //         trace!("Checking view {} can_focus={}", name, view.can_focus());
        //         if view.can_focus() {
        //             self.focused_view = Some(name.clone());
        //             view.focus();
        //             return Some(Event::redraw(None).code);
        //             // return view
        //             //     .handle_event(&Event {
        //             //         code: event.clone(),
        //             //         target: None,
        //             //     })
        //             //     .map(|e| e.code);
        //         }
        //     }
        // }

        // if let Some(view) = &self
        //     .focused_view
        //     .as_mut()
        //     .and_then(|id| self.views.get_mut(id))
        // {
        //     return view
        //         .handle_event(&Event {
        //             code: EventCode::Tab,
        //             target: None,
        //         })
        //         .map(|e| e.code);
        // }
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
