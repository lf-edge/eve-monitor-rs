// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{model::model::Model, traits::IWindow, ui::action::UiActions};

use super::{
    action::Action,
    widgets::{button::ButtonElement, label::LabelElement},
    window::Window,
};

struct MessageBoxState {
    content: String,
}

// ─── shared render ────────────────────────────────────────────────────────────

fn do_render(
    w: &mut Window<MessageBoxState>,
    _rect: &Rect,
    frame: &mut Frame<'_>,
    _model: &Rc<Model>,
) {
    let frame_rect = w.get_layout("frame");
    frame.render_widget(Clear {}, frame_rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black))
        .title(format!(" {} ", &w.name));

    frame.render_widget(block, frame_rect);
}

fn on_key_event(w: &mut Window<MessageBoxState>, key: KeyEvent) -> Option<Action> {
    if key.code == KeyCode::Esc {
        return Some(Action::new(&w.name, UiActions::DismissDialog));
    }
    None
}

fn on_child_ui_action(
    w: &mut Window<MessageBoxState>,
    source: &String,
    action: &UiActions,
) -> Option<Action> {
    debug!("message_box on_child_ui_action: {}:{:?}", source, action);
    match action {
        UiActions::ButtonClicked(_) => Some(Action::new(&w.name, UiActions::DismissDialog)),
        _ => None,
    }
}

// ─── dismissable message box (OK + Cancel) ────────────────────────────────────

fn on_init(w: &mut Window<MessageBoxState>) {
    w.add_widget("label", LabelElement::new(w.state.content.clone()));
    w.add_widget("ok", ButtonElement::new("OK"));
    w.add_widget("cancel", ButtonElement::new("Cancel"));
    w.set_focus_tracker_tab_order(vec!["ok", "cancel"]);
}

fn do_layout(w: &mut Window<MessageBoxState>, rect: &Rect, _model: &Rc<Model>) {
    debug!("message_box do_layout");
    let rect = crate::ui::tools::centered_rect_fixed(50, 14, *rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    w.update_layout("frame", rect);

    let [content_area, buttons_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::End)
            .areas(inner);

    w.update_layout("label", content_area);

    let [_, ok, cancel, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(1),
    ])
    .areas(buttons_area);

    w.update_layout("ok", ok);
    w.update_layout("cancel", cancel);
}

pub fn create_message_box(window_caption: &str, content: &str) -> impl IWindow {
    Window::builder(window_caption)
        .with_on_init(on_init)
        .with_layout(do_layout)
        .with_render(do_render)
        .with_on_key_event(on_key_event)
        .with_on_child_ui_action(on_child_ui_action)
        .with_state(MessageBoxState { content: content.to_string() })
        .build()
        .unwrap()
}

// ─── OK-only info box ─────────────────────────────────────────────────────────

fn info_on_init(w: &mut Window<MessageBoxState>) {
    w.add_widget("label", LabelElement::new(w.state.content.clone()));
    w.add_widget("ok", ButtonElement::new("OK"));
    w.set_focus_tracker_tab_order(vec!["ok"]);
}

fn info_do_layout(w: &mut Window<MessageBoxState>, rect: &Rect, _model: &Rc<Model>) {
    debug!("info_box do_layout");
    let rect = crate::ui::tools::centered_rect_fixed(52, 16, *rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    w.update_layout("frame", rect);

    let [content_area, buttons_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::End)
            .areas(inner);

    w.update_layout("label", content_area);

    let [_, ok, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(6),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(buttons_area);

    w.update_layout("ok", ok);
}

/// A single-button informational popup. Use for notices where there is nothing to cancel.
pub fn create_info_box(window_caption: &str, content: &str) -> impl IWindow {
    Window::builder(window_caption)
        .with_on_init(info_on_init)
        .with_layout(info_do_layout)
        .with_render(do_render)
        .with_on_key_event(on_key_event)
        .with_on_child_ui_action(on_child_ui_action)
        .with_state(MessageBoxState { content: content.to_string() })
        .build()
        .unwrap()
}

// ─── non-dismissable system popup ────────────────────────────────────────────

/// Creates a non-dismissable system message box (no buttons, ignores all key input).
/// Must be removed programmatically by the UI/layer stack (for example, by removing this window from the stack).
pub fn create_system_message_box(window_caption: &str, content: &str) -> impl IWindow {
    fn sys_on_init(w: &mut Window<MessageBoxState>) {
        w.add_widget("label", LabelElement::new(w.state.content.clone()));
    }

    fn sys_do_layout(w: &mut Window<MessageBoxState>, rect: &Rect, _model: &Rc<Model>) {
        let rect = crate::ui::tools::centered_rect_fixed(44, 7, *rect);
        let content_area = rect.inner(Margin { horizontal: 1, vertical: 1 });
        w.update_layout("frame", rect);
        w.update_layout("label", content_area);
    }

    fn sys_on_key_event(_w: &mut Window<MessageBoxState>, _key: KeyEvent) -> Option<Action> {
        None // swallow all keys — cannot be dismissed by the user
    }

    Window::builder(window_caption)
        .with_on_init(sys_on_init)
        .with_layout(sys_do_layout)
        .with_render(do_render)
        .with_on_key_event(sys_on_key_event)
        .with_state(MessageBoxState { content: content.to_string() })
        .build()
        .unwrap()
}
