use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use log::info;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Text, ToSpan},
    widgets::{Cell, HighlightSpacing, Row, TableState},
    Frame,
};

use crate::{
    events::Event,
    tpm::diff::{ConfigFileStatus, InterpretedTpmEvent},
    traits::{IEventHandler, IPresenter, IWindow},
};

use super::traits::ISelector;

trait TpmEventDecode {
    fn short_description(&self) -> Line;
    fn diff(&self) -> (Vec<Row<'_>>, Vec<Row<'_>>);
}

pub struct TpmEventList {
    state: TableState,
    size: usize,
}

pub struct VaultPage {
    list: TpmEventList,
}

impl IWindow for VaultPage {}

impl VaultPage {
    pub fn new() -> impl IWindow {
        VaultPage {
            list: TpmEventList {
                state: TableState::default(),
                size: 0,
            },
        }
    }

    fn render_event_list<'a, 'b>(
        &'a mut self,
        area: &'a Rect,
        frame: &mut Frame<'_>,
        bar: &str,
        tpm_events: &'b Vec<(u32, Vec<InterpretedTpmEvent>)>,
    ) -> Option<&'b InterpretedTpmEvent> {
        info!("render_event_list: {:?}", tpm_events);
        // get selected event and return it
        let selected_event = self
            .list
            .state
            .selected()
            .map(|selected| {
                let selected_event = tpm_events.iter().map(|e| &e.1).flatten().nth(selected);
                selected_event
            })
            .flatten();

        let rows = tpm_events
            .iter()
            .map(|(pcr, events)| {
                events
                    .iter()
                    .map(|event| {
                        let cells = vec![
                            Cell::from(pcr.to_string()).green(),
                            Cell::from(event.short_description()),
                        ];
                        ratatui::widgets::Row::new(cells)
                    })
                    .collect::<Vec<ratatui::widgets::Row>>()
            })
            .flatten()
            .collect::<Vec<ratatui::widgets::Row>>();

        self.list.size = rows.len();

        let width = &[Constraint::Length(4), Constraint::Fill(1)];

        let header = Row::new(vec!["PCR", "Dsscription"])
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        let table = ratatui::widgets::Table::new(rows, width)
            .header(header)
            .block(
                ratatui::widgets::Block::default()
                    .title("TPM Events")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .column_spacing(1)
            .highlight_symbol(Text::from(vec![
                // "".into(),
                bar.into(),
                bar.into(),
                bar.into(),
                bar.into(),
                // "".into(),
            ]))
            .highlight_spacing(HighlightSpacing::Always)
            .row_highlight_style(
                ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
            );

        frame.render_stateful_widget(table, *area, &mut self.list.state);
        selected_event
    }

    fn render_event_details(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        bar: &str,
        tpm_events: &InterpretedTpmEvent,
    ) {
    }
}

impl ISelector for VaultPage {
    type Item = usize;
    fn select_next(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            if selected < self.list.size - 1 {
                self.list.state.select(Some(selected + 1));
            }
        } else {
            self.list.state.select(Some(0));
        }
    }

    fn select_previous(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let index = selected.saturating_sub(1);
            self.list.state.select(Some(index));
        }
    }

    fn select_first(&mut self) {
        self.list.state.select(Some(0));
    }

    fn select_last(&mut self) {
        let index = self.list.size.saturating_sub(1);

        self.list.state.select(Some(index));
    }

    fn selected(&self) -> Option<usize> {
        self.list.state.selected()
    }
}

impl IPresenter for VaultPage {
    fn render(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        model: &std::rc::Rc<crate::model::model::Model>,
        _focused: bool,
    ) {
        let bar = " █ ";
        let model = model.borrow();
        info!("Rendering VaultPage");
        if let Some(tpm_events) = model.tpm_log_parse_result.as_ref() {
            info!("Loags not empty");
            // create  a layout
            let [top_part, user_tips] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(*area);

            let [pcr_event_list_rect, decoding_rect] =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .areas(top_part);

            let g_m = is_grub_cfg_modified(tpm_events);
            if let Some(selected) =
                self.render_event_list(&pcr_event_list_rect, frame, bar, tpm_events)
            {
                self.render_event_details(&decoding_rect, frame, bar, &selected);
            }
        }
    }
}

fn is_grub_cfg_modified(tpm_events: &Vec<(u32, Vec<InterpretedTpmEvent>)>) -> bool {
    let grub_fg_file_modified = tpm_events
        .iter()
        .filter_map(|e| if e.0 == 8 { Some(&e.1) } else { None })
        .flatten()
        .any(|event| {
            if let InterpretedTpmEvent::ConfigFileModified { file, .. } = event {
                file == "/config/grub.cfg"
            } else {
                false
            }
        });
    grub_fg_file_modified
}

impl IEventHandler for VaultPage {
    fn handle_event(&mut self, event: Event) -> Option<super::action::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Home | KeyCode::PageUp if key.modifiers == KeyModifiers::CONTROL => {
                    self.select_first()
                }
                KeyCode::End | KeyCode::PageDown if key.modifiers == KeyModifiers::CONTROL => {
                    self.select_last()
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}

impl<'a> Into<Line<'a>> for ConfigFileStatus {
    fn into(self) -> Line<'a> {
        match self {
            ConfigFileStatus::Modified => "modified".yellow().into(),
            ConfigFileStatus::Deleted => "deleted".red().into(),
            ConfigFileStatus::Added => "created".green().into(),
        }
    }
}

impl TpmEventDecode for InterpretedTpmEvent {
    fn short_description(&self) -> Line {
        match self {
            InterpretedTpmEvent::ConfigFileModified { file, status } => {
                Line::default().spans(vec![format!("{}: ", file).white(), status.to_span()])
            }
            InterpretedTpmEvent::KernelCmdLineModified { old: _, new: _ } => {
                Line::from("Kernel command line modified")
            }
            InterpretedTpmEvent::GrubCfgModified => Line::from("Grub settings were changed"),
            InterpretedTpmEvent::BootOrderModified { old: _, new: _ } => {
                Line::from("Boot order was changed")
            }
            InterpretedTpmEvent::BootOptionsModified { old: _, new: _ } => {
                Line::from("Boot options were changed")
            }
            InterpretedTpmEvent::Error(tpm_event) => {
                Line::from(format!("Error decoding event: {:?}", tpm_event))
            }
            InterpretedTpmEvent::EnterBios => Line::from("BIOS Setup enter"),
        }
    }

    fn diff(&self) -> (Vec<Row<'_>>, Vec<Row<'_>>) {
        todo!()
    }
}
