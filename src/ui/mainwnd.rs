use std::{collections::HashMap, fmt::Debug};

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

use super::focus_tracker::{FocusMode, FocusTracker};

struct WindowBuilder {}
pub struct MainWnd {
    pub ft: FocusTracker,
    pub widgets: HashMap<String, Box<dyn IWidget>>,
    pub layout: HashMap<String, Rect>,
}

impl Debug for MainWnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MainWnd");
        // print the layout
        for (k, v) in &self.layout {
            write!(f, "layout: {} => {:#?}", k, v);
        }
        // print focus tracker
        write!(f, "focus tracker: {:#?}", &self.ft);
        Ok(())
    }
}

impl MainWnd {
    // fn add_radio_group(&mut self, labels: Vec<String>, title: String) {
    //     let mut rg = RadioGroupState {
    //         labels: labels.clone(),
    //         selected: 0,
    //         title,
    //     };
    //     let widget = RadioGroupWidget {};
    //     self.rg = RadioGroupView {
    //         state: rg,
    //         widget,
    //         ft: FocusTracker::create_from_taborder(labels, None, FocusMode::Wrap),
    //     };
    // }
    fn add_widget<W: StatefulWidgetRef>(&mut self, widget: W) {
        todo!()
    }
}

impl IWindow for MainWnd {}
impl IEventDispatcher for MainWnd {
    fn dispatch_event(&self, event: UiCommand) {
        todo!()
    }
}
impl IEventHandler for MainWnd {
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

impl IFocusTracker for MainWnd {
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
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(&focused_view).unwrap();
            widget.clear_focus();
        }
        let ret = loop {
            let next = self.ft.focus_prev();
            if let Some(focused_view) = next {
                let widget = self.widgets.get_mut(&focused_view).unwrap();
                if widget.is_focus_tracker() {
                    widget.set_focus();
                    break Some(focused_view);
                }
            }
        };
        ret
    }

    fn get_focused_view_name(&self) -> Option<String> {
        self.ft.get_focused_view()
    }
}

impl IVisible for MainWnd {}
impl IFocusAcceptor for MainWnd {}
impl IPresenter for MainWnd {
    fn do_layout(
        &mut self,
        area: &Rect,
    ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
        let mut layout = HashMap::new();
        let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
        for (i, col) in cols.iter().enumerate() {
            let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
            for (j, row) in rows.iter().enumerate() {
                let area_name = format!("{}-{}", i, j);
                layout.insert(area_name, *row);
            }
        }
        self.layout = layout.clone();
        info!("do_layout: MainWnd {:#?}", &self.layout);
        layout
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        info!("rendering: MainWnd {:#?}", &self);
        let r = self.layout.get("0-0").unwrap();
        let rg = self.widgets.get_mut("RadioGroup").unwrap();
        rg.render(r, frame);

        let r = self.layout.get("0-1").unwrap();
        let rg = self.widgets.get_mut("RadioGroup 1").unwrap();
        rg.render(r, frame);

        let r = self.layout.get("3-3").unwrap();
        let rg = self.widgets.get_mut("Label").unwrap();
        rg.render(r, frame);

        let r = self.layout.get("3-0").unwrap();
        let rg = self.widgets.get_mut("Input").unwrap();
        rg.render(r, frame);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
