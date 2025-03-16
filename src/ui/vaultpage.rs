use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};

use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyModifiers};
use log::info;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Text, ToSpan},
    widgets::{Cell, HighlightSpacing, Paragraph, Row, TableState, Wrap},
    Frame,
};

use crate::{
    efi::vars::EfiLoadOption,
    events::Event,
    ipc::eve_types::EfiVariable,
    lcs::DiffOp,
    model::model::MonitorModel,
    tpm::{
        diff::{ConfigFileStatus, InterpretedTpmEvent, InterpretedTpmEventRef},
        tcg_events::{TcgEfiActionEvent, TcgIPLEvent},
        tcg_tpmlog::{TcgRawTpmEvent, TcgTpmEventType},
        tpmlog::TpmEventRef,
    },
    traits::{IEventHandler, IPresenter, IWindow},
};

use super::traits::{ISelectable, ISelector};

trait TpmEventDecode {
    fn short_description(&self) -> Line;
    fn diff(&self) -> (Vec<Row<'_>>, Vec<Row<'_>>);
}

pub struct TpmEventList {
    state: TableState,
    size: usize,
    page_size: usize,
}

impl ISelectable for TpmEventList {
    type Item = usize;

    fn current_index(&self) -> Option<usize> {
        self.state.selected()
    }

    fn selection_size(&self) -> usize {
        self.size
    }

    fn select(&mut self, index: usize) {
        self.state.select(Some(index));
    }

    fn selected_item(&self) -> Option<Self::Item> {
        self.current_index()
    }
}

pub struct VaultPage {
    user_mode_list: TpmEventList,
    expert_list_old: TpmEventList,
    expert_list_new: TpmEventList,
    expert_mode: bool,
    expert_selected_old: bool,
    expert_diff_only: bool,
}

impl VaultPage {
    fn get_page_size(&self) -> usize {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.page_size
            } else {
                self.expert_list_new.page_size
            }
        } else {
            self.user_mode_list.page_size
        }
    }
}

impl IWindow for VaultPage {}

fn get_boot_efi_var_description(index: u16, vars: &Vec<EfiVariable>) -> Result<String> {
    let var_name = format!("Boot{:04x}", index);
    let var = vars
        .iter()
        .find(|v| v.name == var_name)
        .ok_or_else(|| anyhow!("Variable {} not found", var_name))?;
    let load_options = EfiLoadOption::try_from(var.value.as_slice())?;
    Ok(load_options.description)
}

fn data_omitted_message(t: &TcgTpmEventType) -> String {
    format!("<data omitted>")
}

fn decoding_error_message(t: &TcgTpmEventType, e: String) -> String {
    format!("Error decoding event: {} : {}", t, e)
}

fn decode_tcg_tpm_event(event: &TcgRawTpmEvent) -> String {
    match event.event_type {
        TcgTpmEventType::PrebootCert => data_omitted_message(&event.event_type),
        TcgTpmEventType::PostCode => data_omitted_message(&event.event_type),
        TcgTpmEventType::NoAction => data_omitted_message(&event.event_type),
        TcgTpmEventType::Separator => "".to_string(),
        TcgTpmEventType::Action => TcgEfiActionEvent::try_from(event)
            .map(|e| e.get().to_string())
            .unwrap_or_else(|a| data_omitted_message(&event.event_type)),
        TcgTpmEventType::EventTag => data_omitted_message(&event.event_type),
        TcgTpmEventType::SCRTMContents => data_omitted_message(&event.event_type),
        TcgTpmEventType::SCRTMVersion => data_omitted_message(&event.event_type),
        TcgTpmEventType::CPUMicrocode => data_omitted_message(&event.event_type),
        TcgTpmEventType::PlatformConfigFlags => data_omitted_message(&event.event_type),
        TcgTpmEventType::TableOfDevices => data_omitted_message(&event.event_type),
        TcgTpmEventType::CompactHash => data_omitted_message(&event.event_type),
        TcgTpmEventType::IPL => TcgIPLEvent::try_from(event)
            .map(|e| e.get().to_string())
            .unwrap_or_else(|e| data_omitted_message(&event.event_type)),
        TcgTpmEventType::IPLPartitionData => data_omitted_message(&event.event_type),
        TcgTpmEventType::NonhostCode => data_omitted_message(&event.event_type),
        TcgTpmEventType::NonhostConfig => data_omitted_message(&event.event_type),
        TcgTpmEventType::NonhostInfo => data_omitted_message(&event.event_type),
        TcgTpmEventType::OmitBootDeviceEvents => data_omitted_message(&event.event_type),
        TcgTpmEventType::PostCode2 => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiEventBase => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiVariableDriverConfig => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiVariableBoot => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiBootServicesApplication => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiBootServicesDriver => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiRuntimeServicesDriver => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiGPTEvent => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiAction => TcgEfiActionEvent::try_from(event)
            .map(|e| e.get().to_string())
            .unwrap_or_else(|a| data_omitted_message(&event.event_type)),
        TcgTpmEventType::EfiPlatformFirmwareBlob => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiHandoffTables => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiPlatformFirmwareBlob2 => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiHandoffTables2 => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiVariableBoot2 => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiGPTEvent2 => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiHCRTMEvent => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiVariableAuthority => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiSPDMFirmwareBlob => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiSPDMFirmwareConfig => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiSPDMDevicePolicy => data_omitted_message(&event.event_type),
        TcgTpmEventType::EfiSPDMDeviceAuthority => data_omitted_message(&event.event_type),
    }
}

impl VaultPage {
    pub fn new() -> impl IWindow {
        VaultPage {
            user_mode_list: TpmEventList {
                state: TableState::default(),
                size: 0,
                page_size: 1,
            },
            expert_list_old: TpmEventList {
                state: TableState::default(),
                size: 0,
                page_size: 1,
            },
            expert_list_new: TpmEventList {
                state: TableState::default(),
                size: 0,
                page_size: 1,
            },
            expert_mode: false,
            expert_selected_old: true,
            expert_diff_only: false,
        }
    }

    fn render_event_list<'a, 'b>(
        &'a mut self,
        area: &'a Rect,
        frame: &mut Frame<'_>,
        bar: &str,
        tpm_events: &'b Vec<InterpretedTpmEventRef>,
    ) -> Option<&'b InterpretedTpmEventRef> {
        info!("render_event_list: {:?}", tpm_events);

        // get selected event and return it
        let selected_event = self
            .user_mode_list
            .state
            .selected()
            .map(|selected| {
                let selected_event = tpm_events.iter().nth(selected);
                selected_event
            })
            .flatten();

        let rows = tpm_events
            .iter()
            .map(|event| {
                let cells = vec![
                    Cell::from(event.pcr.to_string()).green(),
                    Cell::from(event.event.short_description()),
                ];
                ratatui::widgets::Row::new(cells)
            })
            .collect::<Vec<ratatui::widgets::Row>>();

        self.user_mode_list.size = rows.len();

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

        frame.render_stateful_widget(table, *area, &mut self.user_mode_list.state);
        selected_event
    }

    // fn render_event_details(
    //     &mut self,
    //     area: &Rect,
    //     frame: &mut Frame<'_>,
    //     bar: &str,
    //     tpm_event: &InterpretedTpmEventRef,
    //     old_good_log: &EveTpmLog,
    //     new_log: &EveTpmLog,
    // ) {
    //     match &tpm_event.event {
    //         InterpretedTpmEvent::ConfigFileModified { file, status } => todo!(),
    //         InterpretedTpmEvent::KernelCmdLineModified { old, new } => {
    //             //create a flex layout with 2 paragraphs
    //             let [old_rect, new_rect] =
    //                 Layout::horizontal([Constraint::Max(5), Constraint::Max(5)])
    //                     .flex(Flex::Start)
    //                     .areas(*area);
    //             // just display 2 paragpaths with wrapped text for old and new kernel command line
    //             let old = Paragraph::new(old.as_str())
    //                 .wrap(Wrap { trim: true })
    //                 .block(
    //                     ratatui::widgets::Block::default()
    //                         .title("Old kernel command line")
    //                         .borders(ratatui::widgets::Borders::ALL),
    //                 );

    //             let new = Paragraph::new(new.as_str())
    //                 .wrap(Wrap { trim: true })
    //                 .block(
    //                     ratatui::widgets::Block::default()
    //                         .title("New kernel command line")
    //                         .borders(ratatui::widgets::Borders::ALL),
    //                 );

    //             frame.render_widget(old, old_rect);
    //             frame.render_widget(new, new_rect);
    //         }
    //         InterpretedTpmEvent::GrubCfgModified => {
    //             //just a paragraph with block explaining that /config/grub.cfg was modified by user
    //             let w = Paragraph::new("file /config/grub.cfg was changed by user")
    //                 .wrap(Wrap { trim: true })
    //                 .block(
    //                     ratatui::widgets::Block::default()
    //                         .title("Event details")
    //                         .borders(ratatui::widgets::Borders::ALL),
    //                 );
    //             frame.render_widget(w, *area);
    //         }
    //         InterpretedTpmEvent::BootOrderModified { old, new } => {
    //             let old_vars = old_good_log.efi_vars.as_ref().unwrap();
    //             let new_vars = new_log.efi_vars.as_ref().unwrap();
    //             let old = old
    //                 .iter()
    //                 .map(|i| {
    //                     // look for variable with name and get display string
    //                     get_boot_efi_var_description(*i, &old_vars)
    //                 })
    //                 .collect::<Vec<_>>();

    //             let new = new
    //                 .iter()
    //                 .map(|i| {
    //                     // look for variable with name and get display string
    //                     get_boot_efi_var_description(*i, &new_vars)
    //                 })
    //                 .collect::<Vec<_>>();

    //             let mut rows = vec![];

    //             for (i, (old, new)) in old.iter().zip(new.iter()).enumerate() {
    //                 rows.push(Row::new(vec![
    //                     Cell::from(i.to_string()).green(),
    //                     Cell::from(old.as_ref().unwrap().as_str()),
    //                     Cell::from(new.as_ref().unwrap().as_str()),
    //                 ]));
    //             }

    //             let width = &[
    //                 Constraint::Length(4),
    //                 Constraint::Percentage(50),
    //                 Constraint::Percentage(50),
    //             ];

    //             let header = Row::new(vec!["#", "Expected", "Current"])
    //                 .style(Style::default().fg(Color::Yellow))
    //                 .height(1);

    //             let table = ratatui::widgets::Table::new(rows, width)
    //                 .header(header)
    //                 .block(
    //                     ratatui::widgets::Block::default()
    //                         .title("Event details")
    //                         .borders(ratatui::widgets::Borders::ALL),
    //                 )
    //                 .column_spacing(1)
    //                 .highlight_symbol(Text::from(vec![
    //                     // "".into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     // "".into(),
    //                 ]))
    //                 .highlight_spacing(HighlightSpacing::Always)
    //                 .row_highlight_style(
    //                     ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
    //                 );

    //             frame.render_widget(table, *area);
    //         }
    //         InterpretedTpmEvent::BootOptionsModified { old, new } => todo!(),
    //         InterpretedTpmEvent::EnterBios => {
    //             let rows = vec![Row::new(vec![
    //                 Cell::from("1"),
    //                 Cell::from("User entered BIOS setup"),
    //             ])];

    //             let width = &[Constraint::Length(4), Constraint::Fill(1)];

    //             let header = Row::new(vec!["#", "Description"])
    //                 .style(Style::default().fg(Color::Yellow))
    //                 .height(1);

    //             let table = ratatui::widgets::Table::new(rows, width)
    //                 .header(header)
    //                 .block(
    //                     ratatui::widgets::Block::default()
    //                         .title("Event details")
    //                         .borders(ratatui::widgets::Borders::ALL),
    //                 )
    //                 .column_spacing(1)
    //                 .highlight_symbol(Text::from(vec![
    //                     // "".into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     bar.into(),
    //                     // "".into(),
    //                 ]))
    //                 .highlight_spacing(HighlightSpacing::Always)
    //                 .row_highlight_style(
    //                     ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
    //                 );

    //             frame.render_widget(table, *area);
    //         }
    //         InterpretedTpmEvent::Error => {
    //             // // find original TPM events using indexes
    //             // let old_event = old_good_log
    //             //     .log
    //             //     .events
    //             //     .get(tpm_event.old_original_index)
    //             //     .unwrap();

    //             // let w = Paragraph::new(decoding_error_message(&old_event.event, tpm_event.error))
    //             //     .wrap(Wrap { trim: true })
    //             //     .block(
    //             //         ratatui::widgets::Block::default()
    //             //             .title("Event details")
    //             //             .borders(ratatui::widgets::Borders::ALL),
    //             //     );

    //             // frame.render_widget(table, *area);
    //         }
    //     }
    // }

    fn render_user_mode(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        model: &Rc<RefCell<MonitorModel>>,
    ) {
        let bar = " █ ";
        let model = model.borrow();
        info!("Rendering VaultPage");
        // if let Some(tpm_events) = model.tpm_log_parse_result.as_ref() {
        //     let old_good_log = model.old_good_tpm_log.as_ref().unwrap();
        //     let new_log = model.new_tpm_log.as_ref().unwrap();

        //     info!("Logs not empty");
        //     // create  a layout
        //     let [top_part, user_tips] =
        //         Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
        //             .areas(*area);

        //     let [pcr_event_list_rect, decoding_rect] =
        //         Layout::horizontal([Constraint::Length(40), Constraint::Fill(1)]).areas(top_part);

        //     let g_m = is_grub_cfg_modified(tpm_events);
        //     if let Some(selected) =
        //         self.render_event_list(&pcr_event_list_rect, frame, bar, tpm_events)
        //     {
        //         self.render_event_details(
        //             &decoding_rect,
        //             frame,
        //             bar,
        //             &selected,
        //             old_good_log,
        //             new_log,
        //         );
        //     }
        // }
    }

    fn render_expert_mode(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        model: &RefCell<MonitorModel>,
    ) {
        let model = model.borrow();
        let [top_part, bottom_part] =
            Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]).areas(*area);

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(top_part);
        self.expert_list_old.page_size = left.height as usize;
        self.expert_list_new.page_size = right.height as usize;

        self.expert_list_old.size = model
            .tpm
            .as_ref()
            .unwrap()
            .diff_ops_old
            .as_ref()
            .unwrap()
            .len();
        self.expert_list_new.size = model
            .tpm
            .as_ref()
            .unwrap()
            .diff_ops_new
            .as_ref()
            .unwrap()
            .len();

        Self::render_tpm_log_expert(
            left,
            frame,
            &model.tpm.as_ref().unwrap().parsed_old.as_ref().unwrap(),
            &model.tpm.as_ref().unwrap().diff_ops_old.as_ref().unwrap(),
            "Expected",
            self.expert_selected_old,
            &mut self.expert_list_old.state,
            self.expert_diff_only,
        );
        Self::render_tpm_log_expert(
            right,
            frame,
            &model.tpm.as_ref().unwrap().parsed_new.as_ref().unwrap(),
            &model.tpm.as_ref().unwrap().diff_ops_new.as_ref().unwrap(),
            "Current",
            !self.expert_selected_old,
            &mut self.expert_list_new.state,
            self.expert_diff_only,
        );
    }

    fn cell_for_event_op<'a, 'b>(event: &'a TpmEventRef, op: &'a DiffOp) -> Cell<'b> {
        match op {
            DiffOp::Unchanged(_) => Cell::from("=".to_string()),
            DiffOp::Add(_) => Cell::from("+".to_string()),
            DiffOp::Del(_) => Cell::from("-".to_string()),
            DiffOp::Mod(_, _) => Cell::from("~".to_string()),
        }
    }

    fn style_for_event_op<'a, 'b>(op: &'a DiffOp) -> Style {
        match op {
            DiffOp::Unchanged(_) => Style::default().fg(Color::White),
            DiffOp::Add(_) => Style::default().fg(Color::Green),
            DiffOp::Del(_) => Style::default().fg(Color::Red),
            DiffOp::Mod(_, _) => Style::default().fg(Color::Blue),
        }
    }

    pub fn row_for_tpm_event_expert<'a, 'b>(
        event: &'a TpmEventRef,
        op: &'a DiffOp,
    ) -> ratatui::widgets::Row<'b> {
        let style = Self::style_for_event_op(op);
        let cells = vec![
            Cell::from(event.pcr.to_string()).style(style),
            Cell::from(event.event.event_type().to_string()).style(style),
            Self::cell_for_event_op(event, op).style(style),
        ];
        ratatui::widgets::Row::new(cells)
    }

    fn render_tpm_log_expert(
        area: Rect,
        frame: &mut Frame<'_>,
        evelt_log: &Vec<TpmEventRef>,
        edit_ops: &Vec<DiffOp>,
        caption: &str,
        selected: bool,
        list_state: &mut TableState,
        diff_only: bool,
    ) {
        let bar = " █ ";
        let tpm_events = &evelt_log;
        let rows = tpm_events
            .iter()
            .zip(edit_ops.iter())
            .filter(|(_, op)| {
                if diff_only {
                    !matches!(op, DiffOp::Unchanged(_))
                } else {
                    true
                }
            })
            .map(|(event, op)| Self::row_for_tpm_event_expert(event, op))
            .collect::<Vec<ratatui::widgets::Row>>();

        let width = &[
            Constraint::Length(2),
            Constraint::Percentage(20),
            Constraint::Fill(1),
        ];

        let header = Row::new(vec!["PCR", "Event", "Data"])
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        let table = ratatui::widgets::Table::new(rows, width)
            .header(header)
            .block(
                ratatui::widgets::Block::default()
                    .title(format!("{} TPM Events", caption))
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(if selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    }),
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

        frame.render_stateful_widget(table, area, list_state);
    }
}

impl ISelector for VaultPage {
    type Item = usize;
    fn select_next(&mut self) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.select_next();
            } else {
                self.expert_list_new.select_next();
            }
        } else {
            self.user_mode_list.select_next();
        }
    }

    fn select_previous(&mut self) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.select_previous();
            } else {
                self.expert_list_new.select_previous();
            }
        } else {
            self.user_mode_list.select_previous();
        }
    }

    fn select_first(&mut self) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.select(0);
            } else {
                self.expert_list_new.select(0);
            }
        } else {
            self.user_mode_list.select(0);
        }
    }

    fn select_last(&mut self) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old
                    .select(self.expert_list_old.selection_size().saturating_sub(1));
            } else {
                self.expert_list_new
                    .select(self.expert_list_new.selection_size().saturating_sub(1));
            }
        } else {
            self.user_mode_list
                .select(self.user_mode_list.selection_size().saturating_sub(1));
        }
    }

    fn selected(&self) -> Option<usize> {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.selected()
            } else {
                self.expert_list_new.selected()
            }
        } else {
            self.user_mode_list.selected()
        }
    }

    fn select_forward_by(&mut self, count: usize) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.select_forward_by(count);
            } else {
                self.expert_list_new.select_forward_by(count);
            }
        } else {
            self.user_mode_list.select_forward_by(count);
        }
    }

    fn select_backward_by(&mut self, count: usize) {
        if self.expert_mode {
            if self.expert_selected_old {
                self.expert_list_old.select_backward_by(count);
            } else {
                self.expert_list_new.select_backward_by(count);
            }
        } else {
            self.user_mode_list.select_backward_by(count);
        }
    }
}

impl IPresenter for VaultPage {
    fn render(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        model: &Rc<crate::model::model::Model>,
        _focused: bool,
    ) {
        if self.expert_mode {
            self.render_expert_mode(area, frame, model);
        } else {
            self.render_user_mode(area, frame, model);
        }
    }
}

fn is_grub_cfg_modified(tpm_events: &Vec<InterpretedTpmEventRef>) -> bool {
    let grub_fg_file_modified = tpm_events
        .iter()
        .filter_map(|e| if e.pcr == 8 { Some(&e.event) } else { None })
        .any(|event| {
            if let InterpretedTpmEvent::ConfigFileModified { file, .. } = &event {
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
                KeyCode::PageUp => {
                    self.select_backward_by(self.get_page_size());
                }
                KeyCode::PageDown => {
                    self.select_forward_by(self.get_page_size());
                }
                KeyCode::F(12) => {
                    self.expert_mode = !self.expert_mode;
                    if self.expert_mode {
                        self.expert_selected_old = true;
                    }
                }
                KeyCode::F(2) => {
                    self.expert_diff_only = !self.expert_diff_only;
                }
                KeyCode::Tab => {
                    if self.expert_mode {
                        self.expert_selected_old = !self.expert_selected_old;
                    }
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
            InterpretedTpmEvent::Error => {
                Line::from(format!("Error decoding event:")) // {:?}", tpm_event))
            }
            InterpretedTpmEvent::EnterBios => Line::from("BIOS Setup enter"),
        }
    }

    fn diff(&self) -> (Vec<Row<'_>>, Vec<Row<'_>>) {
        todo!()
    }
}
