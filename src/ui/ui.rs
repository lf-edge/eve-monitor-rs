// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    model::device::network::NetworkInterfaceStatus,
    traits::{IPresenter, IWindow},
    ui::{input_dialog::create_input_dialog, ipdialog::create_ip_dialog},
};
use core::fmt::Debug;
use crossterm::event::{KeyCode, KeyModifiers};
use log::debug;
use ratatui::{
    layout::{
        Constraint::{Fill, Length},
        Layout,
    },
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, Tabs, Widget},
};
use std::rc::Rc;
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    events::Event,
    model::{device::dmesg::DmesgViewer, model::Model},
    terminal::TerminalWrapper,
    traits::IEventHandler,
    ui::action::UiActions,
};

use super::{
    action::Action,
    app_page::ApplicationsPage,
    evalstatus_page::EvalStatusPage,
    layer_stack::LayerStack,
    networkpage::create_network_page,
    reboot_warning::{create_reboot_warning_dialog, WINDOW_NAME as REBOOT_WARNING_NAME},
    statusbar::{create_status_bar, StatusBarState},
    summary_page::SummaryPage,
    vaultpage::VaultPage,
    window::Window,
};

const STARTUP_WARNING_NAME: &str = "Evaluation Mode";
const CONNECTION_POPUP_NAME: &str = "EVE Connection";

#[cfg(debug_assertions)]
use super::homepage::HomePage;

use std::result::Result::Ok;

use anyhow::Result;

pub struct Ui {
    pub terminal: TerminalWrapper,
    pub action_tx: UnboundedSender<Action>,
    pub views: Vec<LayerStack>,
    pub selected_tab: UiTabs,
    pub status_bar: Window<StatusBarState>,
    first_frame: bool,
    startup_warning_shown: bool,    // Whether startup warning was ever triggered
    startup_warning_visible: bool,  // Whether startup warning is currently on all tab stacks
    last_reboot_warning: u64,       // Last countdown value seen (u64::MAX = uninitialized)
    reboot_warning_shown: bool,     // Whether reboot warning is currently on all tab stacks
    connection_popup_shown: bool,   // Whether IPC connection popup is on all tab stacks
    last_connection_message: Option<String>, // Last message shown in the connection popup
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
pub enum UiTabs {
    #[default]
    Summary,
    #[cfg(debug_assertions)]
    Home,
    Network,
    Applications,
    Vault,
    Dmesg,
    #[strum(to_string = "Eval Status")]
    EvalStatus,
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ui :)")
    }
}

impl Ui {
    pub fn new(action_tx: UnboundedSender<Action>, terminal: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            terminal,
            action_tx,
            views: vec![LayerStack::new(); UiTabs::COUNT],
            selected_tab: UiTabs::default(),
            status_bar: create_status_bar(),
            first_frame: true,
            startup_warning_shown: false,
            startup_warning_visible: false,
            last_reboot_warning: u64::MAX,
            reboot_warning_shown: false,
            connection_popup_shown: false,
            last_connection_message: None,
        })
    }

    fn tabs() -> Tabs<'static> {
        let tab_titles = UiTabs::iter().map(UiTabs::to_tab_title);
        let block = Block::new().title(" Use ctrl + ◄ ► to change tab");
        Tabs::new(tab_titles)
            .block(block)
            .highlight_style(Modifier::REVERSED)
            .divider(" ")
            .padding("", "")
    }

    pub fn init(&mut self) {
        self.views[UiTabs::Summary as usize].push(Box::new(SummaryPage::new()));
        #[cfg(debug_assertions)]
        {
            self.views[UiTabs::Home as usize].push(Box::new(HomePage::new()));
        }

        self.views[UiTabs::Network as usize].push(Box::new(create_network_page()));

        self.views[UiTabs::Applications as usize].push(Box::new(ApplicationsPage::new()));
        self.views[UiTabs::Dmesg as usize].push(Box::new(DmesgViewer::new()));
        self.views[UiTabs::Vault as usize].push(Box::new(VaultPage::new()));
        self.views[UiTabs::EvalStatus as usize].push(Box::new(EvalStatusPage::new()));
    }

    pub fn draw(&mut self, model: Rc<Model>) {
        let screen_layout = Layout::vertical([Length(3), Fill(0), Length(3)]);
        let tabs_widget = Ui::tabs();
        let git_version = model.borrow().app_version.clone();

        //TODO: handle terminal event
        let _ = self.terminal.draw(|frame| {
            let area = frame.area();
            let [top_bar_rect, body_rect, statusbar_rect] = screen_layout.areas(area);

            if self.first_frame {
                self.first_frame = false;
                frame.render_widget(Clear, area);
            }

            let [tabs_rect, version_rect] =
                Layout::horizontal([Fill(0), Length(git_version.len() as u16)]).areas(top_bar_rect);

            let version_widget = Paragraph::new(git_version.clone()).fg(Color::DarkGray);
            frame.render_widget(version_widget, version_rect);

            tabs_widget
                .select(self.selected_tab as usize)
                .render(tabs_rect, frame.buffer_mut());

            // redraw from the bottom up
            let stack = &mut self.views[self.selected_tab as usize];
            let last_index = stack.len().saturating_sub(1);
            for (index, layer) in stack.iter_mut().enumerate() {
                layer.render(&body_rect, frame, &model, index == last_index);
            }
            // Read the hint after render so that pages which update their
            // cached state inside render() (e.g. EvalStatusPage) return a
            // current value rather than one frame behind.
            {
                let hint = stack.last_mut().and_then(|top| top.status_bar_tips());
                debug!("Hint: {:?}", hint);
                model.borrow_mut().status_bar_tips = hint;
            }
            // render status bar
            self.status_bar
                .render(&statusbar_rect, frame, &model, false);
        });
    }

    fn invalidate(&mut self) {
        self.action_tx
            .send(Action::new("app", UiActions::Redraw))
            .unwrap();
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Action> {
        if event != Event::Tick {
            debug!("Ui handle_event {:?}", event);
        }

        match event {
            // only for debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('e'))
                    && (key.modifiers == KeyModifiers::CONTROL)
                    && cfg!(debug_assertions) =>
            {
                debug!("CTRL+q: application Quit requested");
                self.action_tx
                    .send(Action::new("user", UiActions::Quit))
                    .unwrap();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('r'))
                    && (key.modifiers == KeyModifiers::CONTROL)
                    && cfg!(debug_assertions) =>
            {
                debug!("CTRL+r: manual Redraw requested");
                self.invalidate();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('p'))
                    && (key.modifiers == KeyModifiers::CONTROL)
                    && cfg!(debug_assertions) =>
            {
                debug!("CTRL+p: manual layer.pop() requested");
                self.pop_layer();
            }

            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('a'))
                    && (key.modifiers == KeyModifiers::CONTROL)
                    && cfg!(debug_assertions) =>
            {
                debug!("CTRL+a: manual panic requested");
                panic!("Manual panic requested");
            }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(action) = self.views[self.selected_tab as usize]
                    .last_mut()?
                    .handle_event(Event::Key(key))
                {
                    match action.action {
                        UiActions::DismissDialog => {
                            // Only dismiss global warnings if the dismiss originated from
                            // the corresponding warning window; otherwise, dismiss the
                            // actual top-most layer.
                            let mut handled = false;

                            if self.reboot_warning_shown {
                                if action.source == REBOOT_WARNING_NAME {
                                    self.dismiss_reboot_warning();
                                    handled = true;
                                }
                            }

                            if !handled && self.startup_warning_visible {
                                if action.source == STARTUP_WARNING_NAME {
                                    self.dismiss_startup_warning();
                                    handled = true;
                                }
                            }

                            if !handled {
                                self.pop_layer();
                            }
                        }

                        _ => {
                            return Some(action);
                        }
                    }
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left {
                    debug!("CTRL+Left: switching tab view");
                    self.selected_tab = self.selected_tab.previous();
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right {
                    debug!("CTRL+Right: switching tab view");
                    self.selected_tab = self.selected_tab.next();
                }
            }
            Event::Tick => {
                // forward tick event to all layers. Collect actions
                for layer in self.views[self.selected_tab as usize].iter_mut() {
                    if let Some(action) = layer.handle_event(Event::Tick) {
                        self.action_tx.send(action).unwrap();
                    }
                }
                // and to the status bar
                self.status_bar.handle_event(Event::Tick);
            }
            _ => {
                debug!("Unhandled event: {:?}", event);
            }
        }

        None
    }

    fn push_layer(&mut self, d: impl IWindow + 'static) {
        self.views[self.selected_tab as usize].push(Box::new(d))
    }

    pub fn pop_layer(&mut self) -> Option<Box<dyn IWindow>> {
        self.views[self.selected_tab as usize].pop()
    }

    pub fn show_ip_dialog(&mut self, iface: NetworkInterfaceStatus) {
        let d = create_ip_dialog(&iface);
        self.push_layer(d);
    }

    pub fn show_server_url_dialog(&mut self, url: &str) {
        let d = create_input_dialog(
            "Change server URL",
            "Server URL",
            url,
            "https://prod.zedcontrol.zededa.net",
        );
        self.push_layer(d);
    }

    pub fn message_box(&mut self, title: &str, message: &str) {
        let d = super::message_box::create_message_box(title, message);
        self.push_layer(d);
    }

    pub fn show_eval_startup_warning(&mut self) {
        if !self.startup_warning_shown {
            self.startup_warning_shown = true;
            self.startup_warning_visible = true;
            let title = STARTUP_WARNING_NAME;
            let message = "This device is running in EVALUATION MODE.\n\
\n\
The device will reboot several times as part of\n\
automated evaluation testing.\n\
\n\
You may inspect all status tabs freely, but:\n\
\n\
  DO NOT change any settings\n\
  DO NOT manually reboot the device\n\
\n\
Check the Eval Status tab for the reboot countdown.";
            // Push to every tab so the warning is visible regardless of which tab is active.
            for stack in self.views.iter_mut() {
                let d = super::message_box::create_info_box(title, message);
                stack.push(Box::new(d));
            }
        }
    }

    /// Remove the startup warning from every tab's layer stack.
    fn dismiss_startup_warning(&mut self) {
        if self.startup_warning_visible {
            for stack in self.views.iter_mut() {
                stack.remove_by_name(STARTUP_WARNING_NAME);
            }
            self.startup_warning_visible = false;
        }
    }

    pub fn check_and_show_reboot_warning(&mut self, model: &Rc<Model>) {
        if let Some(eval_status) = &model.borrow().eval_status {
            let countdown = eval_status.reboot_countdown;

            // countdown == 0 means no reboot scheduled; dismiss any visible warning.
            if countdown == 0 {
                self.last_reboot_warning = u64::MAX;
                self.dismiss_reboot_warning();
                return;
            }

            // Show warning once when the countdown crosses below 5 minutes.
            // Detect threshold crossing: previously >= 300, now < 300.
            if countdown < 300 && self.last_reboot_warning >= 300 && !self.reboot_warning_shown {
                self.reboot_warning_shown = true;
                // Push to every tab so the warning is visible regardless of which tab is active.
                for stack in self.views.iter_mut() {
                    let d = create_reboot_warning_dialog(countdown);
                    stack.push(Box::new(d));
                }
            } else if countdown < 60 && self.last_reboot_warning >= 60 {
                // Refresh (or re-show) at the urgent threshold so the border turns red and
                // the message changes from "within 5 minutes" to "REBOOT IMMINENT".
                self.reboot_warning_shown = true;
                for stack in self.views.iter_mut() {
                    stack.remove_by_name(REBOOT_WARNING_NAME);
                    let d = create_reboot_warning_dialog(countdown);
                    stack.push(Box::new(d));
                }
            }
            self.last_reboot_warning = countdown;
        } else {
            // eval_status is gone (e.g. IPC disconnect): clear any visible warning.
            self.last_reboot_warning = u64::MAX;
            self.dismiss_reboot_warning();
        }
    }

    /// Remove the reboot warning from every tab's layer stack by name.
    fn dismiss_reboot_warning(&mut self) {
        if self.reboot_warning_shown {
            for stack in self.views.iter_mut() {
                stack.remove_by_name(REBOOT_WARNING_NAME);
            }
            self.reboot_warning_shown = false;
        }
    }

    /// Push a non-dismissable system popup onto every tab's layer stack.
    pub fn show_connection_popup(&mut self, message: &str) {
        if self.connection_popup_shown {
            // Skip the replace entirely if the message hasn't changed.
            if self.last_connection_message.as_deref() == Some(message) {
                return;
            }
            // Popup already shown: replace it on each stack so the message stays accurate.
            for stack in self.views.iter_mut() {
                stack.remove_by_name(CONNECTION_POPUP_NAME);
                let popup =
                    super::message_box::create_system_message_box(CONNECTION_POPUP_NAME, message);
                stack.push(Box::new(popup));
            }
        } else {
            // Popup not yet shown: create it on each stack and mark it as visible.
            for stack in self.views.iter_mut() {
                let popup =
                    super::message_box::create_system_message_box(CONNECTION_POPUP_NAME, message);
                stack.push(Box::new(popup));
            }
            self.connection_popup_shown = true;
        }
        self.last_connection_message = Some(message.to_string());
    }

    /// Remove the connection popup from every tab's layer stack by name.
    pub fn dismiss_connection_popup(&mut self) {
        if !self.connection_popup_shown {
            return;
        }
        for stack in self.views.iter_mut() {
            stack.remove_by_name(CONNECTION_POPUP_NAME);
        }
        self.connection_popup_shown = false;
        self.last_connection_message = None;
    }
}

impl UiTabs {
    fn to_tab_title(self) -> Line<'static> {
        let text = self.to_string();
        format!(" {text} ").bg(Color::Black).into()
    }

    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}
