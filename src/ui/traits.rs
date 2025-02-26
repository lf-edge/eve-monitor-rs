// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use ratatui::style::Style;

pub trait IntoRatatuiStyle {
    fn style(&self) -> Style;
}
pub trait ISelector {
    fn select_next(&mut self);
    fn select_previous(&mut self);
    fn select_first(&mut self);
    fn select_last(&mut self);
    fn selected(&self) -> Option<String>;
}
