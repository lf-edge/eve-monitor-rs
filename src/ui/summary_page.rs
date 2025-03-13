// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use crossterm::event::{KeyCode, KeyModifiers};
use log::{debug, info};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text, ToText},
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::{
    events::Event,
    ipc::eve_types::{AttestState, ZedAgentStatus},
    model::model::{Model, OnboardingStatus, VaultStatus},
    tpm::diff::InterpretedTpmEvent,
    traits::{IEventHandler, IPresenter, IWindow},
    ui::action::{Action, UiActions},
};

#[derive(Default)]
pub struct SummaryPage {
    attestation_state: String,
    last_attest_error: String,
}

impl SummaryPage {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn set_attestation_status(&mut self, z: &ZedAgentStatus) {
        match z.attest_state {
            AttestState::StateNone => {
                self.attestation_state = "Attestation not yet started".into();
                self.last_attest_error = "".into();
            }
            AttestState::StateAttestWait
            | AttestState::StateAttestEscrowWait
            | AttestState::StateInternalQuoteWait => {
                self.attestation_state = "Attestation in progress...".into();
            }
            AttestState::StateRestartWait => {
                self.attestation_state = "Attestation Restarted...".into();
                if !z.attest_error.is_empty() && self.last_attest_error != z.attest_error {
                    self.last_attest_error = z.attest_error.clone();
                }
            }
            AttestState::StateComplete => {
                self.attestation_state = "Complete".into();
                self.last_attest_error = "".into();
            }
            _ => {
                if !z.attest_error.is_empty() && self.last_attest_error != z.attest_error {
                    self.last_attest_error = z.attest_error.clone();
                }
            }
        }
    }

    pub fn update_attestation_state(&mut self, model: &Rc<Model>) {
        let model = model.borrow();
        let vault_status = &model.vault_status;

        if !vault_status.is_vault_locked() {
            self.attestation_state = String::new();
            self.last_attest_error = String::new();
            return;
        }

        if let Some(z) = &model.z_status {
            self.set_attestation_status(z);
        }
    }
}

impl IWindow for SummaryPage {}

impl IEventHandler for SummaryPage {
    fn handle_event(&mut self, event: crate::events::Event) -> Option<super::action::Action> {
        // handle Ctrl+s to change the server
        match event {
            Event::Key(key)
                if (key.code == KeyCode::Char('s')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+s: server change requested");
                return Some(Action::new("net", UiActions::ChangeServer));
            }
            _ => {}
        }
        None
    }
}

impl IPresenter for SummaryPage {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        self.update_attestation_state(model);

        let [server, onboarding_status_and_app_sunnary_rect, vault_status_rect, pcr_decoder_rect] =
            Layout::vertical(vec![
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .areas(*area);

        let [onboarding_status_rect, app_summary_rect] =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(onboarding_status_and_app_sunnary_rect);

        let server_url = ratatui::widgets::Paragraph::new(
            model
                .borrow()
                .node_status
                .server
                .clone()
                .unwrap_or("N/A".to_string()),
        )
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Server (CTRL+s to change)"),
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        frame.render_widget(server_url, server);

        self.render_onboarding_status(model, frame, onboarding_status_rect);
        self.render_app_summary(model, frame, app_summary_rect);

        self.render_vault_status(model, frame, vault_status_rect);
        self.render_pcr_decoder(model, frame, pcr_decoder_rect);
    }
}

impl SummaryPage {
    fn render_onboarding_status(
        &self,
        model: &Rc<Model>,
        frame: &mut Frame<'_>,
        onboarding_status_rect: Rect,
    ) {
        let onboarding_status = model.borrow().node_status.onboarding_status.clone();
        let mut text = Vec::new();
        let mut spans = vec![];
        spans.push(Span::styled("status: ", Style::default().fg(Color::White)));
        spans.push(match onboarding_status {
            OnboardingStatus::Unknown => {
                Span::styled("Checking...", Style::default().fg(Color::Yellow))
            }
            OnboardingStatus::Onboarding => {
                Span::styled("Onboarding...", Style::default().fg(Color::Yellow))
            }
            OnboardingStatus::Onboarded(_) => {
                Span::styled("Onboarded", Style::default().fg(Color::Green))
            }
            OnboardingStatus::Error(_) => Span::styled("Error", Style::default().fg(Color::Red)),
        });

        text.push(Line::from(spans));

        match onboarding_status {
            OnboardingStatus::Unknown => {
                text.push(Line::from(vec![
                    Span::styled("GUID: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Yellow)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Green)),
                ]));
            }
            OnboardingStatus::Onboarding => {
                text.push(Line::from(vec![
                    Span::styled("GUID: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Yellow)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Green)),
                ]));
            }
            OnboardingStatus::Onboarded(guid) => {
                text.push(Line::from(vec![
                    Span::styled("GUID: ", Style::default().fg(Color::White)),
                    Span::styled(format!("{}", guid), Style::default().fg(Color::White)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Green)),
                ]));
            }
            OnboardingStatus::Error(err) => {
                text.push(Line::from(vec![
                    Span::styled("GUID: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Yellow)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled(err, Style::default().fg(Color::Red)),
                ]));
            }
        }

        let onboarding_status = ratatui::widgets::Paragraph::new(Text::from(text))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Onboarding status"),
            )
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        frame.render_widget(onboarding_status, onboarding_status_rect);
    }

    fn render_app_summary(&self, model: &Rc<Model>, frame: &mut Frame<'_>, app_summary_rect: Rect) {
        let apps = &model.borrow().node_status.app_summary;

        let mut app_summary_text = vec![];
        app_summary_text.push(Line::from(vec![
            Span::raw("Running:  "),
            Span::styled(
                format!("{}", apps.total_running),
                Style::default().fg(Color::Green),
            ),
        ]));
        app_summary_text.push(Line::from(vec![
            Span::raw("Starting: "),
            Span::styled(
                format!("{}", apps.total_starting),
                Style::default().fg(Color::Green),
            ),
        ]));
        app_summary_text.push(Line::from(vec![
            Span::raw("Stopping: "),
            Span::styled(
                format!("{}", apps.total_stopping),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        app_summary_text.push(Line::from(vec![
            Span::raw("In error: "),
            Span::styled(
                format!("{}", apps.total_error),
                Style::default().fg(Color::Red),
            ),
        ]));
        let app_summary = ratatui::widgets::Paragraph::new(Text::from(app_summary_text))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("App summary"),
            )
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        frame.render_widget(app_summary, app_summary_rect);
    }

    fn render_vault_status(&self, model: &Rc<Model>, frame: &mut Frame<'_>, status_rect: Rect) {
        let model = model.borrow();
        let vault_status = &model.vault_status;
        let z_status = &model.z_status;
        let mut text = Vec::new();
        let mut spans = vec![];
        spans.push(Span::styled("Status: ", Style::default().fg(Color::White)));
        spans.push(match vault_status {
            VaultStatus::Unknown => Span::styled("Unknown", Style::default().fg(Color::Yellow)),
            VaultStatus::EncryptionDisabled(_, _) => {
                Span::styled("Encryption disabled", Style::default().fg(Color::Yellow))
            }
            VaultStatus::Unlocked(_) => Span::styled("Unlocked", Style::default().fg(Color::Green)),
            VaultStatus::Locked(_, _) => Span::styled("Locked", Style::default().fg(Color::Red)),
        });

        text.push(Line::from(spans));

        match vault_status {
            VaultStatus::Unknown => {
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Green)),
                ]));
            }
            VaultStatus::EncryptionDisabled(reason, tpm_used) => {
                text.push(Line::from(vec![
                    Span::styled("TPM used: ", Style::default().fg(Color::White)),
                    if *tpm_used {
                        Span::styled("Yes", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("No", Style::default().fg(Color::Red))
                    },
                ]));
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red)),
                    Span::styled(&reason.error, Style::default().fg(Color::White)),
                ]));
            }
            VaultStatus::Unlocked(tpm_used) => {
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::White)),
                    Span::styled("N/A", Style::default().fg(Color::Green)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("TPM used: ", Style::default().fg(Color::White)),
                    if *tpm_used {
                        Span::styled("Yes", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("No", Style::default().fg(Color::Red))
                    },
                ]));
            }
            VaultStatus::Locked(err, pcr) => {
                text.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red)),
                    Span::styled(&err.error, Style::default().fg(Color::White)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Affected PCRs: ", Style::default().fg(Color::White)),
                    if let Some(pcr) = pcr {
                        Span::styled(format!("{:?}", pcr), Style::default().fg(Color::Green))
                    } else {
                        Span::styled("N/A", Style::default().fg(Color::Yellow))
                    },
                ]));
                // look at attestation status
                // Basically we need to
                // 1. show last attestation error
                // 2. attestation will go through following states Wait -> InternalQuoteWait -> RestartWait -> Complete
                // and some other. Show minimal information to the user
                text.push(Line::from(vec![
                    Span::styled("Attest: ", Style::default().fg(Color::Red)),
                    Span::styled(&self.attestation_state, Style::default().fg(Color::White)),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Attest error: ", Style::default().fg(Color::Red)),
                    Span::styled(&self.last_attest_error, Style::default().fg(Color::White)),
                ]));
            }
        }

        match z_status {
            Some(status) => {
                text.push(Line::from(vec![
                    Span::styled("Attestation status: ", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{:#?}", status.attest_state),
                        Style::default().fg(Color::Green),
                    ),
                ]));
                text.push(Line::from(vec![
                    Span::styled("Attestation error: ", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{:#?}", status.attest_error),
                        Style::default().fg(Color::Green),
                    ),
                ]));
                // is maintenance mode enabled
                text.push(Line::from(vec![
                    Span::styled("Maintenance mode: ", Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{:#?}", status.maintenance_mode),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }
            None => {}
        }

        let vault_status_widget = ratatui::widgets::Paragraph::new(Text::from(text))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Vault status"),
            )
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        frame.render_widget(vault_status_widget, status_rect);
    }

    fn table_pcr14<'a>(data: &'a Vec<InterpretedTpmEvent>) -> (Table<'a>, u16) {
        let width = [Constraint::Length(10), Constraint::Length(10)];
        let raws = data
            .iter()
            .filter_map(|e| match e {
                InterpretedTpmEvent::ConfigFileModified { file, status } => {
                    Some(Row::new(vec![Text::from(file.clone()), status.to_text()]))
                }
                // InterpretedTpmEvent::Error(tpm_event) => todo!(),
                _ => None,
            })
            .collect::<Vec<Row<'_>>>();
        let height = raws.len() as u16 + 4;
        let table = Table::new(raws, width)
            .header(
                Row::new(vec!["File", "Status"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("PCR 14"),
            )
            .widths(&width)
            .style(Style::default().fg(Color::White));
        (table, height)
    }
    fn table_pcr8<'a>(data: &'a Vec<InterpretedTpmEvent>) -> (Table<'a>, u16) {
        let width = [Constraint::Length(10), Constraint::Length(10)];
        let raws = data
            .iter()
            .filter_map(|e| match e {
                InterpretedTpmEvent::KernelCmdLineModified { old, new } => Some(Row::new(vec![
                    Cell::new(old.as_str()),
                    Cell::new(new.as_str()),
                ])),
                // InterpretedTpmEvent::Error(tpm_event) => todo!(),
                _ => None,
            })
            .collect::<Vec<Row<'_>>>();
        let height = raws.len() as u16 + 4;
        let table = Table::new(raws, width)
            .header(
                Row::new(vec!["Old value", "new value"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Grub configuration modified"),
            )
            .widths(&width)
            .style(Style::default().fg(Color::White));
        (table, height)
    }

    // using ratauil
    fn render_pcr_decoder(&self, model: &Rc<Model>, frame: &mut Frame<'_>, status_rect: Rect) {
        let model = model.borrow();
        let vault_status = &model.vault_status;
        let tpm = &model.tpm_log_parse_result;
        let mut tables = Vec::new();
        let mut heights = Vec::new();
        if vault_status.is_vault_locked() {
            if let Some(tpm) = tpm {
                for e in tpm {
                    match e {
                        (14, events) => {
                            let (table, height) = Self::table_pcr14(events);
                            tables.push(table);
                            heights.push(Constraint::Length(height));
                        }
                        (8, events) => {
                            let (table, height) = Self::table_pcr8(events);
                            tables.push(table);
                            heights.push(Constraint::Length(height));
                        }
                        _ => {}
                    }
                }
                // create a layout
                let layout = Layout::vertical(heights).margin(1);
                let layout = layout.split(status_rect);
                for (table, rect) in tables.into_iter().zip(layout.into_iter()) {
                    frame.render_widget(table, *rect);
                }
            }
        }
    }
}
