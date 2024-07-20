use crate::events;
use crate::traits::IElementEventHandler;
use crate::ui::activity::Activity;
use std::{cell::RefCell, fmt::Debug};

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

pub type WidgetMap = ElementHashMap<Box<dyn IWidget>>;
pub type LayoutMap = ElementHashMap<Rect>;

pub type LayoutFn = Box<dyn FnMut(&Rect) -> Option<LayoutMap>>;
pub type RenderFn = Box<dyn FnMut(&Rect, &mut ratatui::Frame<'_>)>;

pub struct WindowBuilder<D> {
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

    on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,

    state: Option<D>,
}

impl<D> WindowBuilder<D> {
    pub fn widget<S: Into<String>>(mut self, name: S, widget: Box<dyn IWidget>) -> Self {
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
        F: FnMut(&Rect, &mut ratatui::Frame<'_>) + 'static,
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
        F: FnMut(Action, &mut D) -> Option<UiActions> + 'static,
    {
        self.on_action = Some(Box::new(on_action));
        self
    }

    pub fn with_state(mut self, state: D) -> Self {
        self.state = Some(state);
        self
    }

    pub fn build(self) -> Result<Window<D>> {
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
            self.do_layout,
            self.do_render,
            self.on_action,
            self.state.unwrap(),
        ))
    }
}

pub struct Window<D> {
    pub v: VisualState,
    pub name: String,
    pub ft: FocusTracker,
    pub widgets: ElementHashMap<Box<dyn IWidget>>,
    pub layout: ElementHashMap<Rect>,
    pub do_layout: Option<LayoutFn>,
    pub do_render: Option<RenderFn>,
    pub on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,
    pub state: RefCell<D>,
}

impl<S> Debug for Window<S> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<D> Window<D> {
    pub(self) fn new<S: Into<String>>(
        name: S,
        ft: FocusTracker,
        widgets: WidgetMap,
        do_layout: Option<LayoutFn>,
        do_render: Option<RenderFn>,
        on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,
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

    pub fn builder<S: Into<String>>(name: S) -> WindowBuilder<D> {
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

impl<D> IWindow for Window<D> {}

impl<D> IEventHandler for Window<D> {
    fn handle_event(&mut self, key: events::Event) -> Option<Action> {
        match key {
            events::Event::Key(key) => {
                if let Some(activity) = self.ft.handle_key_event(key) {
                    match activity {
                        Activity::Action(action) => {
                            return Some(Action::new(self.name.clone(), action))
                        }
                        Activity::Event(_) => {}
                    }
                }
                // forward the event to the focused view
                if let Some(focused_view) = self.ft.get_focused_view() {
                    let widget = self.widgets.get_mut(&focused_view).unwrap();
                    if let Some(activity) = widget.handle_key_event(key) {
                        match activity {
                            Activity::Action(action) => {
                                if let Some(on_action) = self.on_action.as_mut() {
                                    if let Some(new_action) = on_action(
                                        Action {
                                            source: focused_view,
                                            action,
                                            target: None,
                                        },
                                        &mut self.state.borrow_mut(),
                                    ) {
                                        return Some(Action::new(self.name.clone(), new_action));
                                    }
                                }
                            }
                            Activity::Event(event) => {
                                return self.ft.handle_key_event(event).and_then(|act| match act {
                                    Activity::Action(action) => {
                                        Some(Action::new(self.name.clone(), action))
                                    }
                                    Activity::Event(_) => None,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl<D> IFocusTracker for Window<D> {
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
                            widget.set_focus(true);
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
                            widget.set_focus(true);
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

impl<D> IVisible for Window<D> {}
impl<D> IFocusAcceptor for Window<D> {
    fn set_focus(&mut self, focus: bool) {
        self.v.focused = focus;
        // set focus on focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            if let Some(widget) = self.widgets.get_mut(&focused_view) {
                widget.set_focus(true);
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
impl<D> IPresenter for Window<D> {
    // fn do_layout(
    //     &mut self,
    //     area: &Rect,
    // ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
    //     (self.do_layout)(area).unwrap();
    //     //TODO: do we need upper layer to know about the layout? probably not
    //     HashMap::new()
    // }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>, focused: bool) {
        if let Some(custom_render) = &mut self.do_render {
            (custom_render)(area, frame)
        };

        let focused_widget = if focused {
            self.ft.get_focused_view().or(Some("".to_string())).unwrap()
        } else {
            "".to_string()
        };
        if let Some(layouter) = &mut self.do_layout {
            let layout = (layouter)(area).unwrap();

            self.widgets.iter_mut().for_each(|(name, widget)| {
                layout.get(name).and_then(|rect| {
                    widget.render(rect, frame, *name == focused_widget);
                    None::<D>
                });
            });
        } else {
            self.widgets
                .iter_mut()
                .for_each(|(name, widget)| widget.render(area, frame, *name == focused_widget));
        }

        // let r = layout.get("0-0").unwrap();
        // let rg = widgets.get_mut("RadioGroup").unwrap();
        // rg.render(r, frame);

        // let r = layout.get("0-1").unwrap();
        // let rg = widgets.get_mut("RadioGroup 1").unwrap();
        // rg.render(r, frame);

        // let r = layout.get("3-3").unwrap();
        // let rg = widgets.get_mut("Label").unwrap();
        // rg.render(r, frame);

        // let r = layout.get("3-0").unwrap();
        // let rg = self.widgets.get_mut("Input").unwrap();
        // // input.render(r, frame);
        // rg.render(r, frame);
        // // frame.render_input_field(input, *r);

        // let r = layout.get("0-2").unwrap();
        // button.render(r, frame);
        // let rg = widgets.get_mut("Button").unwrap();
        // rg.render(r, frame);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
