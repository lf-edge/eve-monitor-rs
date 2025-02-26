// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use crate::events::Event;
use crate::model::model::Model;
use crate::ui::action::{Action, UiActions};
use crate::ui::activity::Activity;
use log::info;
use ratatui::{layout::Rect, Frame};

pub trait IPresenter {
    // fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect>;
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, focused: bool);
    fn can_focus(&self) -> bool {
        true
    }
}

pub trait IVisible {
    fn is_visible(&self) -> bool {
        true
    }
    fn set_visible(&mut self, _visible: bool) {}
}

pub trait IEventHandler {
    fn handle_event(&mut self, _event: Event) -> Option<Action> {
        None
    }
}

pub trait IElementEventHandler {
    fn handle_key_event(&mut self, _key: crossterm::event::KeyEvent) -> Option<UiActions> {
        None
    }
    fn handle_tick(&mut self) -> Option<Activity> {
        None
    }
}

pub trait IWidgetPresenter {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, focused: bool);
    fn can_focus(&self) -> bool {
        true
    }
}

pub trait IWindow: IPresenter + IEventHandler {
    fn on_child_action(&mut self, source: String, action: UiActions) -> Option<Action> {
        info!("Window received child action: {:?} from {}", action, source);
        None
    }
}
pub trait IWidget: IWidgetPresenter + IElementEventHandler {
    fn set_enabled(&mut self, _enabled: bool) {}
    fn is_enabled(&self) -> bool {
        true
    }
}

pub trait IAction: Clone {
    type Target;
    fn get_source(&self) -> &str;
    fn get_target(&self) -> Option<&str>;
    fn split(self) -> (String, Self::Target);
}
