use crate::events;
use crate::traits::IElementEventHandler;
use crate::ui::activity::Activity;
use std::{cell::RefCell, fmt::Debug};

use log::trace;
use ratatui::layout::Rect;

use crate::traits::{IEventHandler, IPresenter, IVisible, IWidget, IWindow};
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
            let tab_order = order
                .clone()
                .into_iter()
                .filter(|name| self.widgets.get(name).is_some_and(|f| f.can_focus()))
                .collect();
            FocusTracker::create_from_taborder(tab_order, self.focused_view, FocusMode::Wrap)
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
    pub widgets: WidgetMap,
    pub layout: LayoutMap,
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
    fn handle_event(&mut self, event: events::Event) -> Option<Action> {
        match event {
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
            events::Event::Tick => {
                // trace!("handle_event {:?}", event);

                // forward to all widgets
                self.widgets.iter_mut().for_each(|(_, widget)| {
                    if let Some(_activity) = widget.handle_tick() {
                        // match activity {
                        //     Activity::Action(action) => {
                        //         if let Some(on_action) = self.on_action.as_mut() {
                        //             if let Some(new_action) = on_action(
                        //                 Action {
                        //                     source: "".to_string(),
                        //                     action,
                        //                     target: None,
                        //                 },
                        //                 &mut self.state.borrow_mut(),
                        //             ) {
                        //                 return Some(Action::new(self.name.clone(), new_action));
                        //             }
                        //         }
                        //     }
                        //     Activity::Event(_) => {}
                        // }
                    }
                });
                return Some(Action::new(self.name.clone(), UiActions::Redraw));
            }
            _ => {}
        }
        None
    }
}

impl<D> IVisible for Window<D> {}
impl<D> IPresenter for Window<D> {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>, focused: bool) {
        if let Some(custom_render) = &mut self.do_render {
            (custom_render)(area, frame)
        };

        let focused_widget = self.ft.get_focused_view().unwrap_or_default();

        if let Some(layouter) = &mut self.do_layout {
            let layout = (layouter)(area).unwrap();

            self.widgets
                .iter_mut()
                .filter_map(|(name, widget)| {
                    layout
                        .get(name)
                        .inspect(|f| trace!("Layout for {}: {:#?}", name, f))
                        .map(|r| (r, widget, *name == focused_widget))
                })
                .for_each(|(rect, widget, w_focused)| {
                    widget.render(rect, frame, w_focused && focused);
                });
        } else {
            self.widgets
                .iter_mut()
                .for_each(|(name, widget)| widget.render(area, frame, *name == focused_widget));
        }
    }

    fn can_focus(&self) -> bool {
        true
    }
}
