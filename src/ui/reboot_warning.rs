// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

pub const WINDOW_NAME: &str = "Reboot Warning";

use std::rc::Rc;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use log::debug;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{model::model::Model, ui::action::UiActions};

use super::{
    action::Action,
    widgets::{button::ButtonElement, label::LabelElement},
    window::Window,
};

#[derive(Debug, Clone)]
pub struct RebootWarningState {
    pub countdown_secs: u64,
}

fn on_init(w: &mut Window<RebootWarningState>) {
    let secs = w.state.countdown_secs;
    let msg = if secs < 60 {
        "⚠  REBOOT IMMINENT  ⚠\n\n\
         The device will reboot imminently as part\n\
         of evaluation testing.\n\n\
         Do not power off the device.\n\
         Check the Eval Status tab for the countdown."
            .to_string()
    } else {
        "⚠  EVALUATION REBOOT WARNING  ⚠\n\n\
         The device will reboot within the next\n\
         5 minutes as part of evaluation testing.\n\n\
         Do not power off the device.\n\
         Check the Eval Status tab for the countdown."
            .to_string()
    };

    w.add_widget("label", LabelElement::new(msg));
    w.add_widget("ok", ButtonElement::new("Acknowledge"));
    w.set_focus_tracker_tab_order(vec!["ok"]);
}

fn do_render(
    w: &mut Window<RebootWarningState>,
    _rect: &Rect,
    frame: &mut Frame<'_>,
    _model: &Rc<Model>,
) {
    let frame_rect = w.get_layout("frame");

    let clear = Clear {};
    frame.render_widget(clear, frame_rect);

    // Use red for urgent reboot warning
    let border_color = if w.state.countdown_secs < 60 {
        Color::Red
    } else {
        Color::Yellow
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black))
        .title(" Evaluation Reboot ");

    frame.render_widget(block, frame_rect);
}

fn do_layout(w: &mut Window<RebootWarningState>, rect: &Rect, _model: &Rc<Model>) {
    debug!("do_layout: reboot warning dialog");

    let rect = crate::ui::tools::centered_rect_fixed(52, 14, *rect);
    let content_with_buttons = rect.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    w.update_layout("frame", rect);

    let [label_area, buttons] =
        Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::End)
            .areas(content_with_buttons);

    w.update_layout("label", label_area);

    // center button
    let [_, button_area, _] = Layout::horizontal(vec![
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(buttons);

    w.update_layout("ok", button_area);
}

fn on_key_event(w: &mut Window<RebootWarningState>, key: KeyEvent) -> Option<Action> {
    debug!("reboot_warning: on_key_event");

    if key.code == KeyCode::Esc {
        return Some(Action::new(&w.name, UiActions::DismissDialog));
    }
    None
}

fn on_child_ui_action(
    w: &mut Window<RebootWarningState>,
    _source: &String,
    action: &UiActions,
) -> Option<Action> {
    debug!("reboot_warning: on_child_ui_action: {:?}", action);
    match action {
        UiActions::ButtonClicked(_name) => {
            Some(Action::new(&w.name, UiActions::DismissDialog))
        }
        _ => None,
    }
}

pub fn create_reboot_warning_dialog(countdown_secs: u64) -> Window<RebootWarningState> {
    let w = Window::builder(WINDOW_NAME)
        .with_on_init(on_init)
        .with_layout(do_layout)
        .with_render(do_render)
        .with_on_key_event(on_key_event)
        .with_on_child_ui_action(on_child_ui_action)
        .with_state(RebootWarningState { countdown_secs })
        .build()
        .unwrap();
    w
}
