use std::rc::Rc;

use crossterm::event::{KeyCode, KeyModifiers};
use log::{debug, info};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    Frame,
};

use crate::{
    events::Event,
    model::model::{Model, OnboardingStatus, VaultStatus},
    traits::{IEventHandler, IPresenter, IWindow},
    ui::action::{Action, UiActions},
};

pub struct SummaryPage {}

impl SummaryPage {
    pub fn new() -> Self {
        Self {}
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
        let [server, onboarding_status_and_app_sunnary_rect, vault_status_rect] =
            Layout::vertical(vec![
                Constraint::Length(3),
                Constraint::Length(6),
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

        render_onboarding_status(model, frame, onboarding_status_rect);
        render_app_summary(model, frame, app_summary_rect);

        render_vault_status(model, frame, vault_status_rect);
    }
}

fn render_onboarding_status(
    model: &Rc<Model>,
    frame: &mut Frame<'_>,
    onboarding_status_rect: Rect,
) {
    let onboarding_status = model.borrow().node_status.onboarding_status.clone();
    let mut text = Vec::new();
    let mut spans = vec![];
    spans.push(Span::styled("status: ", Style::default().fg(Color::White)));
    spans.push(match onboarding_status {
        OnboardingStatus::Unknown => Span::styled("Unknown", Style::default().fg(Color::Yellow)),
        OnboardingStatus::Onboarding => {
            Span::styled("Onboarding...", Style::default().fg(Color::Yellow))
        }
        OnboardingStatus::Onboarded(_) => {
            Span::styled("Onboarded", Style::default().fg(Color::Green))
        }
        OnboardingStatus::Error(_) => Span::styled("Onboarded", Style::default().fg(Color::Red)),
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

    // let status = model.borrow().node_status.onboarding_status.clone();

    let onboarding_status = ratatui::widgets::Paragraph::new(Text::from(text))
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Onboarding status"),
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
    frame.render_widget(onboarding_status, onboarding_status_rect);
}

fn render_app_summary(model: &Rc<Model>, frame: &mut Frame<'_>, app_summary_rect: Rect) {
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

fn render_vault_status(model: &Rc<Model>, frame: &mut Frame<'_>, onboarding_status_rect: Rect) {
    let vault_status = &model.borrow().vault_status;
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
        }
    }

    let vault_status = ratatui::widgets::Paragraph::new(Text::from(text))
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Vault status"),
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
    frame.render_widget(vault_status, onboarding_status_rect);
}
