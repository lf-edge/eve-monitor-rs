use crate::events;
use crate::model::Model;
use crate::ui::activity::Activity;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::{fmt::Debug, rc::Rc};

use log::trace;
use ratatui::layout::Rect;

use crate::traits::{IEventHandler, IPresenter, IVisible, IWidget, IWindow};
use anyhow::Result;

use super::{
    action::{Action, UiActions},
    focus_tracker::{FocusMode, FocusTracker},
};

pub type WidgetMap = HashMap<String, Box<dyn IWidget>>;
pub type LayoutMap = HashMap<String, Rect>;

pub type LayoutFn<D> = Rc<dyn Fn(&mut Window<D>, &Rect, &Rc<Model>)>;
pub type RenderFn<D> = Rc<dyn Fn(&mut Window<D>, &Rect, &mut ratatui::Frame<'_>, &Rc<Model>)>;

pub struct WindowBuilder<D> {
    name: String,
    widgets: WidgetMap,
    // callback for layout
    do_layout: Option<LayoutFn<D>>,
    // callback for rendering
    do_render: Option<RenderFn<D>>,
    // taborder
    tab_order: Option<Vec<String>>,
    // initial focus
    focused_view: Option<String>,

    on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,

    state: Option<D>,
}

impl<D> WindowBuilder<D> {
    pub fn widget<S: Into<String>>(mut self, name: S, widget: impl IWidget + 'static) -> Self {
        self.widgets.insert(name.into(), Box::new(widget));
        self
    }

    pub fn with_layout<F>(mut self, do_layout: F) -> Self
    where
        F: Fn(&mut Window<D>, &Rect, &Rc<Model>) + 'static,
    {
        self.do_layout = Some(Rc::new(do_layout));
        self
    }

    pub fn with_render<F>(mut self, do_render: F) -> Self
    where
        F: Fn(&mut Window<D>, &Rect, &mut ratatui::Frame<'_>, &Rc<Model>) + 'static,
    {
        self.do_render = Some(Rc::new(do_render));
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
        // focused view if set must exist in widgets and taborder if provided
        if let Some(focused_view) = &self.focused_view {
            if !self.widgets.contains_key(focused_view) {
                return Err(anyhow::anyhow!(
                    "Focused view not found in widgets: {}",
                    focused_view
                ));
            }
            if let Some(order) = &self.tab_order {
                if !order.contains(focused_view) {
                    return Err(anyhow::anyhow!(
                        "Focused view not found in tab order: {}",
                        focused_view
                    ));
                }
            }
        }

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
    pub name: String,
    ft: FocusTracker,
    widgets: WidgetMap,
    layout: LayoutMap,
    do_layout: Option<LayoutFn<D>>,
    do_render: Option<RenderFn<D>>,
    on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,
    pub state: D,
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
        do_layout: Option<LayoutFn<D>>,
        do_render: Option<RenderFn<D>>,
        on_action: Option<Box<dyn FnMut(Action, &mut D) -> Option<UiActions>>>,
        state: D,
    ) -> Self {
        Self {
            name: name.into(),
            ft,
            widgets,
            layout: HashMap::new(),
            do_layout,
            do_render,
            on_action,
            state,
        }
    }

    pub fn builder<S: Into<String>>(name: S) -> WindowBuilder<D> {
        WindowBuilder {
            name: name.into(),
            widgets: HashMap::new(),
            do_layout: None,
            do_render: None,
            tab_order: None,
            focused_view: None,
            on_action: None,
            state: None,
        }
    }

    pub fn add_widget<S: Into<String>>(&mut self, name: S, widget: Box<dyn IWidget>) {
        self.widgets.insert(name.into(), widget);
    }

    pub fn update_layout<S: Into<String>>(&mut self, name: S, rect: Rect) {
        self.layout.insert(name.into(), rect);
    }

    pub fn layout<S: Into<String>>(&mut self, name: S) -> Rect {
        self.layout.get(&name.into()).unwrap().clone()
    }

    pub fn render_widget<S: Into<String>>(&mut self, name: S, frame: &mut ratatui::Frame<'_>) {
        let name = name.into();
        let focused = self.ft.get_focused_view().unwrap_or_default() == name;
        let rect = self.layout.get(&name).unwrap().clone();
        let widget = self.widgets.get_mut(&name).unwrap();
        widget.render(&rect, frame, focused);
    }
}

impl<D> IWindow for Window<D> {}

impl<D> IEventHandler for Window<D> {
    fn handle_event(&mut self, event: events::Event) -> Option<Action> {
        match event {
            events::Event::Key(key) => {
                if let Some(action) = self.ft.handle_key_event(key) {
                    return Some(Action::new(self.name.clone(), action));
                }
                // forward the event to the focused view
                let focused_view = self.ft.get_focused_view()?;
                let widget = self.widgets.get_mut(&focused_view).unwrap();
                let activity = widget.handle_key_event(key)?;
                match activity {
                    Activity::Action(action) => {
                        self.on_action.as_mut()?(
                            Action {
                                source: focused_view,
                                action,
                                target: None,
                            },
                            &mut self.state.borrow_mut(),
                        )
                        .and_then(|new_action| Some(Action::new(self.name.clone(), new_action)));
                    }

                    Activity::Event(event) => {
                        return self
                            .ft
                            .handle_key_event(event)
                            .and_then(|act| Some(Action::new(self.name.clone(), act)))
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
    fn render(
        &mut self,
        area: &Rect,
        frame: &mut ratatui::Frame<'_>,
        model: &Rc<Model>,
        focused: bool,
    ) {
        // print layout map
        trace!("Layout: {:#?}", self.layout);

        let focused_widget = self.ft.get_focused_view().unwrap_or_default();

        // always do layout first. New widgets and layout entries may appear
        if let Some(layouter) = self.do_layout.borrow_mut() {
            let layouter = layouter.clone();
            (layouter)(self, area, &model);
        }

        // do custom rendering before we render widgets
        if let Some(custom_render) = self.do_render.borrow_mut() {
            let custom_render = custom_render.clone();
            (custom_render)(self, area, frame, &model)
        };

        let layout = &self.layout;

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
    }

    fn can_focus(&self) -> bool {
        true
    }
}
