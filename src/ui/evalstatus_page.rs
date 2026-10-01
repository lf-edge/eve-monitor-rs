// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use chrono::{DateTime, Datelike, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{
    model::model::Model,
    traits::{IEventHandler, IPresenter, IWindow},
    ui::tools::format_secs,
};

// ─── formatting helpers ────────────────────────────────────────────────────

/// Format nanoseconds into a human-readable time string.
fn format_duration_ns(nanos: u64) -> String {
    let secs = nanos / 1_000_000_000;
    if secs == 0 {
        return "—".to_string();
    }
    format_secs(secs)
}

/// Returns true if `ts` is the Go zero-value (0001-01-01), used as "not set".
fn is_zero_time(ts: &DateTime<chrono::FixedOffset>) -> bool {
    ts.year() == 1
}

/// Parse an RFC-3339 timestamp string and return a "X ago" or "just now" string.
/// Returns "—" for the Go zero-value timestamp. Returns the raw string on parse failure.
fn format_age(timestamp: &str) -> String {
    match DateTime::parse_from_rfc3339(timestamp) {
        Ok(ts) => {
            if is_zero_time(&ts) {
                return "—".to_string();
            }
            let elapsed = Utc::now().signed_duration_since(ts.with_timezone(&Utc));
            let secs = elapsed.num_seconds().max(0) as u64;
            if secs <= 1 {
                "just now".to_string()
            } else {
                format!("{} ago", format_secs(secs))
            }
        }
        Err(_) => timestamp.to_string(),
    }
}

/// Format a test-start timestamp: show date+time and elapsed age side by side.
/// Returns "Not started" for the Go zero-value timestamp.
fn format_start_time(timestamp: &str) -> String {
    match DateTime::parse_from_rfc3339(timestamp) {
        Ok(ts) => {
            if is_zero_time(&ts) {
                return "Not started".to_string();
            }
            let local = ts.with_timezone(&chrono::Local);
            let elapsed = Utc::now().signed_duration_since(ts.with_timezone(&Utc));
            let secs = elapsed.num_seconds().max(0) as u64;
            let age = if secs <= 1 {
                "just now".to_string()
            } else {
                format!("{} ago", format_secs(secs))
            };
            format!("{} ({})", local.format("%Y-%m-%d %H:%M:%S"), age)
        }
        Err(_) => timestamp.to_string(),
    }
}


// ─── page widget ───────────────────────────────────────────────────────────

#[derive(Default)]
pub struct EvalStatusPage {
    /// Cached from the last render; used by status_bar_tips().
    is_eval_platform: bool,
    has_data: bool,
}

impl EvalStatusPage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IWindow for EvalStatusPage {
    fn status_bar_tips(&self) -> Option<String> {
        if !self.has_data {
            Some("Evaluation Status - waiting for data from EVE".to_string())
        } else if self.is_eval_platform {
            Some("Evaluation Status - device is running in evaluation mode".to_string())
        } else {
            Some("Evaluation Status - device is not an evaluation platform".to_string())
        }
    }
}

impl IEventHandler for EvalStatusPage {
    fn handle_event(&mut self, _event: crate::events::Event) -> Option<crate::ui::action::Action> {
        None
    }
}

impl IPresenter for EvalStatusPage {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        let block = Block::default()
            .title(" Evaluation Status ")
            .borders(Borders::ALL);

        let model_ref = model.borrow();

        // Separator line: fills the inner width of the block (outer width minus 2 border columns).
        let sep_width = area.width.saturating_sub(2) as usize;
        let sep = "─".repeat(sep_width.max(1));

        let text = match &model_ref.eval_status {
            // ── No data yet ──────────────────────────────────────────────
            None => {
                self.has_data = false;
                self.is_eval_platform = false;
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Waiting for evaluation status data from EVE…",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            }

            Some(eval_status) => {
                self.has_data = true;
                self.is_eval_platform = eval_status.is_evaluation_platform;

                let mut lines: Vec<Line> = vec![];

                // ── Header banner ─────────────────────────────────────────
                lines.push(Line::from(""));
                if eval_status.is_evaluation_platform {
                    lines.push(Line::from(Span::styled(
                        "  ⚠  EVALUATION MODE ACTIVE  ⚠",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::raw(
                        "This device is running in EVALUATION MODE.",
                    )));
                    lines.push(Line::from(Span::raw(
                        "Device will reboot several times during the test.",
                    )));
                    lines.push(Line::from(Span::styled(
                        "Do not change settings or manually reboot.",
                        Style::default().fg(Color::Yellow),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        "Not an evaluation platform",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )));
                }

                // ── Separator ─────────────────────────────────────────────
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    sep.clone(),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(Span::styled(
                    "Status",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )));

                // ── Status block ──────────────────────────────────────────
                if eval_status.is_evaluation_platform {
                    lines.push(Line::from(vec![
                        Span::raw("Phase:             "),
                        Span::styled(
                            eval_status.phase.as_str(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("Slot:              "),
                        Span::styled(
                            eval_status.current_slot.as_str(),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));
                }

                let onboard_label = if eval_status.allow_onboard {
                    Span::styled("Allowed", Style::default().fg(Color::Green))
                } else if eval_status.is_evaluation_platform {
                    Span::styled(
                        "Blocked (evaluation in progress)",
                        Style::default().fg(Color::Yellow),
                    )
                } else {
                    Span::styled("Blocked", Style::default().fg(Color::DarkGray))
                };
                lines.push(Line::from(vec![
                    Span::raw("Onboarding:        "),
                    onboard_label,
                ]));

                if !eval_status.note.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::raw("Note:              "),
                        Span::styled(
                            eval_status.note.as_str(),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }

                // ── Test information (eval platforms only) ─────────────────
                if eval_status.is_evaluation_platform {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        sep.clone(),
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from(Span::styled(
                        "Test",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )));

                    lines.push(Line::from(vec![
                        Span::raw("Started:           "),
                        Span::styled(
                            format_start_time(eval_status.test_start_time.as_str()),
                            Style::default().fg(Color::Green),
                        ),
                    ]));

                    lines.push(Line::from(vec![
                        Span::raw("Duration:          "),
                        Span::styled(
                            format_duration_ns(eval_status.test_duration),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));

                    // Reboot countdown with context-sensitive label and coloring.
                    let reboot_secs = eval_status.reboot_countdown;
                    if reboot_secs == 0 {
                        lines.push(Line::from(vec![
                            Span::raw("Next Reboot:       "),
                            Span::styled(
                                "Not scheduled",
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    } else if reboot_secs < 60 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "⚠ REBOOT IN:       ",
                                Style::default()
                                    .fg(Color::Red)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format_secs(reboot_secs),
                                Style::default()
                                    .fg(Color::Red)
                                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
                            ),
                        ]));
                    } else if reboot_secs < 300 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "⚠ Reboot in:       ",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format_secs(reboot_secs),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::raw("Next Reboot:       "),
                            Span::styled(
                                format_secs(reboot_secs),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }

                    // ── Inventory (eval platforms only) ───────────────────
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        sep.clone(),
                        Style::default().fg(Color::DarkGray),
                    )));
                    lines.push(Line::from(Span::styled(
                        "Inventory",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )));

                    lines.push(Line::from(vec![
                        Span::raw("Collection:        "),
                        if eval_status.inventory_collected {
                            Span::styled("Complete", Style::default().fg(Color::Green))
                        } else {
                            Span::styled("In Progress", Style::default().fg(Color::Yellow))
                        },
                    ]));

                    // Show directory only while collection is still running.
                    if !eval_status.inventory_collected && !eval_status.inventory_dir.is_empty() {
                        lines.push(Line::from(vec![
                            Span::raw("Directory:         "),
                            Span::styled(
                                eval_status.inventory_dir.as_str(),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    }
                }

                // ── Footer: last updated ───────────────────────────────────
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    sep.clone(),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(vec![
                    Span::raw("Last Updated:      "),
                    Span::styled(
                        format_age(eval_status.last_updated.as_str()),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));

                lines
            }
        };

        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, *area);
    }
}
