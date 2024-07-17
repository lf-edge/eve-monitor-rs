use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use crossterm::event::KeyEvent;
use log::{debug, trace, warn};
use ratatui::layout::Rect;

use crate::traits::{
    IEventHandler, IFocusAcceptor, IFocusTracker, IPresenter, IVisible, IWidget, IWindow,
};
use anyhow::anyhow;
use anyhow::Result;

use super::{
    action::{Action, UiActions},
    focus_tracker::{FocusMode, FocusTracker},
    tools::ElementHashMap,
    widgets::element::VisualState,
};

pub type WidgetMap<A> = ElementHashMap<Box<dyn IWidget<Action = A>>>;
pub type LayoutMap = ElementHashMap<Rect>;

pub type LayoutFn = Box<dyn FnMut(&Rect) -> Option<LayoutMap>>;
pub type RenderFn<A> =
    Box<dyn FnMut(&Rect, &mut ratatui::Frame<'_>, &LayoutMap, &mut WidgetMap<A>)>;

pub struct WindowBuilder<A, D> {
    name: String,
    widgets: WidgetMap<A>,
    // callback for layout
    do_layout: Option<LayoutFn>,
    // callback for rendering
    do_render: Option<RenderFn<A>>,
    // taborder
    tab_order: Option<Vec<String>>,
    // initial focus
    focused_view: Option<String>,

    on_action: Option<Box<dyn FnMut(Action<A>, &mut D) -> Option<UiActions<A>>>>,

    state: Option<D>,
}

impl<A, D> WindowBuilder<A, D> {
    pub fn widget<S: Into<String>>(
        mut self,
        name: S,
        widget: Box<dyn IWidget<Action = A>>,
    ) -> Self {
        self.widgets
            .add(name.into(), widget)
            .expect("Widget name already exists");
        self
    }

    pub fn with_layout<F>(mut self, do_layout: F) -> Self
    where
        F: FnMut(&Rect) -> Option<LayoutMap> + 'static,
    {
        self.do_layout = Some(Box::new(do_layout));
        self
    }

    pub fn with_render<F>(mut self, do_render: F) -> Self
    where
        F: FnMut(&Rect, &mut ratatui::Frame<'_>, &LayoutMap, &mut WidgetMap<A>) + 'static,
    {
        self.do_render = Some(Box::new(do_render));
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

    pub fn on_action<F>(mut self, on_action: F) -> Self
    where
        F: FnMut(Action<A>, &mut D) -> Option<UiActions<A>> + 'static,
    {
        self.on_action = Some(Box::new(on_action));
        self
    }

    pub fn with_state(mut self, state: D) -> Self {
        self.state = Some(state);
        self
    }

    pub fn build(self) -> Result<Window<A, D>> {
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
            self.on_action,
            self.state.unwrap(),
        ))
    }
}

pub struct Window<A, D> {
    pub v: VisualState,
    pub name: String,
    pub ft: FocusTracker,
    pub widgets: ElementHashMap<Box<dyn IWidget<Action = A>>>,
    pub layout: ElementHashMap<Rect>,
    pub do_layout: LayoutFn,
    pub do_render: RenderFn<A>,
    pub on_action: Option<Box<dyn FnMut(Action<A>, &mut D) -> Option<UiActions<A>>>>,
    pub state: RefCell<D>,
}

impl<A, S> Debug for Window<A, S> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<A, D> Window<A, D> {
    pub(self) fn new<S: Into<String>>(
        name: S,
        ft: FocusTracker,
        widgets: WidgetMap<A>,
        do_layout: LayoutFn,
        do_render: RenderFn<A>,
        on_action: Option<Box<dyn FnMut(Action<A>, &mut D) -> Option<UiActions<A>>>>,
        state: D,
    ) -> Self {
        Self {
            name: name.into(),
            ft,
            widgets,
            layout: ElementHashMap::new(),
            do_layout,
            do_render,
            on_action,
            state: RefCell::new(state),
            v: Default::default(),
        }
    }

    pub fn builder<S: Into<String>>(name: S) -> WindowBuilder<A, D> {
        WindowBuilder {
            name: name.into(),
            widgets: ElementHashMap::new(),
            do_layout: None,
            do_render: None,
            tab_order: None,
            focused_view: None,
            on_action: None,
            state: None,
        }
    }
}

impl<A, D> IWindow for Window<A, D> {}

impl<A, D> IEventHandler for Window<A, D> {
    type Action = A;
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action<Self::Action>> {
        // forward the event to the focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(&focused_view).unwrap();
            if let Some(action) = widget.handle_key_event(key) {
                if let Some(on_action) = self.on_action.as_mut() {
                    if let Some(new_action) = on_action(
                        Action {
                            source: focused_view,
                            action,
                            target: None,
                        },
                        &mut self.state.borrow_mut(),
                    ) {
                        return Some(Action {
                            source: self.name.clone(),
                            action: new_action,
                            target: None,
                        });
                    }
                }
            }
        }
        None
    }
}

impl<A, D> IFocusTracker for Window<A, D> {
    fn focus_next(&mut self) -> Option<String> {
        debug!("focus_next: on: {}", &self.name);
        trace!("focus_next: {:#?}", &self.ft);

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
            trace!("Next focused view candidate: {:?}", &next);

            match next {
                Some(focused_view) => {
                    if let Some(widget) = self.widgets.get_mut(&focused_view) {
                        if widget.can_focus() {
                            debug!("setting focus: {}", focused_view);
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
        debug!("focus_prev: on: {}", &self.name);
        trace!("focus_prev: {:#?}", &self.ft);
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
                        if widget.can_focus() {
                            debug!("setting focus: {}", focused_view);
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

impl<A, D> IVisible for Window<A, D> {}
impl<A, D> IFocusAcceptor for Window<A, D> {
    fn set_focus(&mut self) {
        self.v.focused = true;
        // set focus on focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.set_focus();
            }
        }
    }

    fn clear_focus(&mut self) {
        self.v.focused = false;
        // clear focus on focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.clear_focus();
            }
        }
    }

    fn has_focus(&self) -> bool {
        self.v.focused
    }

    fn can_focus(&self) -> bool {
        self.v.can_focus
    }
}
impl<A, D> IPresenter for Window<A, D> {
    // fn do_layout(
    //     &mut self,
    //     area: &Rect,
    // ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
    //     (self.do_layout)(area).unwrap();
    //     //TODO: do we need upper layer to know about the layout? probably not
    //     HashMap::new()
    // }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        (self.do_layout)(area).unwrap();
        (self.do_render)(area, frame, &self.layout, &mut self.widgets);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
