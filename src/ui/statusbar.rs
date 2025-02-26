// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use ratatui::{
    layout::{Constraint, Flex, Layout, Margin},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, WidgetRef},
};

use super::{widgets::label::LabelElement, window::Window};

pub struct StatusBarState {}

pub fn create_status_bar() -> Window<StatusBarState> {
    let clock = LabelElement::new("Clock").on_tick(|label| {
        let now = chrono::Local::now();
        let time = now.format("%H:%M:%S").to_string();
        label.set_text(time);
    });

    let w = Window::builder("StatusBar")
        .with_state(StatusBarState {})
        .widget("Clock", clock)
        .with_layout(|w, rect, _model| {
            let inner_rect = rect.inner(Margin {
                horizontal: 1,
                vertical: 1,
            });

            let layout = Layout::horizontal([Constraint::Length(0), Constraint::Length(8)])
                .flex(Flex::End)
                .split(inner_rect);
            w.update_layout("Clock", layout[1]);
        })
        .with_render(|_w, rect, frame, _model| {
            let blk = Block::new()
                //.border_type(BorderType::Rounded)
                //FIXME: need new Font
                .border_type(BorderType::Plain)
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            blk.render_ref(*rect, frame.buffer_mut());
        })
        .build();

    w.unwrap()
}
