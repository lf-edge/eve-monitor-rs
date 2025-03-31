// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::actions::MonActions;
use crate::events::Event;
use crate::model::model::Model;
use crate::model::model::MonitorModel;
use crate::ui::ipdialog::InterfaceState;
use crate::ui::ui::Ui;

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::result::Result::Ok;
use std::str::FromStr;

use anyhow::Result;
use ipnet::IpNet;
use log::error;
use log::LevelFilter;
use log::{debug, info, trace, warn};

use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use futures::{FutureExt, SinkExt, StreamExt};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::ipc::ipc_client::IpcClient;
use crate::ipc::message::{IpcMessage, Request};
use crate::terminal::TerminalWrapper;
use crate::ui::action::{Action, UiActions};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    config_path: PathBuf,
    pub log_level: String,
}

impl AppConfig {
    fn new<T>(path: T) -> Self
    where
        T: AsRef<std::path::Path>,
    {
        Self {
            config_path: path.as_ref().to_path_buf(),
            log_level: "info".to_string(),
        }
    }

    fn load<T>(path: T) -> Result<Self>
    where
        T: AsRef<std::path::Path>,
    {
        let cfg = std::fs::read_to_string(path)?;
        let cfg: AppConfig = serde_json::from_str(&cfg)?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let cfg = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.config_path, cfg)?;
        Ok(())
    }

    pub fn load_or_create_app_config(base_dir: &Path) -> AppConfig {
        let config_dir = PathBuf::from(base_dir).join("config");
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        let config_path = config_dir.join("config.json");

        AppConfig::load(&config_path).map_or_else(
            |e| {
                log::error!("Failed to load config: {}", e);
                let cfg = AppConfig::new(config_path);
                cfg.save().expect("Failed to save config");
                cfg
            },
            |c| c,
        )
    }
}

pub struct Application {
    terminal_rx: UnboundedReceiver<Event>,
    terminal_tx: UnboundedSender<Event>,
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    ipc_tx: Option<UnboundedSender<IpcMessage>>,
    ui: Ui,
    // this is our model :)
    model: Rc<Model>,
    // pending requests
    pending_requests: HashMap<u64, Rc<dyn Fn(&mut Application)>>,
    config: AppConfig,
}

impl Application {
    pub fn new(config: AppConfig) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
        let (terminal_tx, terminal_rx) = mpsc::unbounded_channel::<Event>();
        let terminal = TerminalWrapper::open_terminal()?;
        let mut ui = Ui::new(action_tx.clone(), terminal)?;
        let model = Rc::new(RefCell::new(MonitorModel::default()));
        let pending_requests = HashMap::new();

        ui.init();

        Ok(Self {
            terminal_rx,
            terminal_tx,
            action_rx,
            action_tx,
            ui,
            ipc_tx: None,
            model,
            pending_requests,
            config,
        })
    }
    pub fn send_ipc_message<F>(&mut self, msg: IpcMessage, handle_response: F)
    where
        F: Fn(&mut Application) -> () + 'static,
    {
        if let Some(ipc_tx) = &self.ipc_tx {
            match &msg {
                IpcMessage::Request { request, id } => {
                    debug!("Pending response for: {:?}", request);
                    self.pending_requests.insert(*id, Rc::new(handle_response));
                }
                _ => {}
            }

            match ipc_tx.send(msg) {
                Ok(_) => {
                    debug!("Sent IPC message");
                }
                Err(e) => {
                    error!("Error sending IPC message: {:?}", e);
                }
            }
        }
    }

    fn get_socket_path() -> String {
        // try to get XDG_RUNTIME_DIR first if we run a standalone app on development host
        if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            return format!("{}/monitor.sock", xdg_runtime_dir);
        } else {
            // EVE path
            return "/run/monitor.sock".to_string();
        }
    }

    fn is_desktop() -> bool {
        //std::env::var("XDG_RUNTIME_DIR").is_ok()
        false
    }

    pub fn handle_ipc_message(&mut self, msg: IpcMessage) {
        match msg {
            IpcMessage::Response { result, id } => {
                debug!("Got response: {:?}", result);
                match result {
                    Ok(_) => {
                        debug!("Response OK");
                        if let Some(handle_response) = self.pending_requests.remove(&id) {
                            handle_response(self);
                        }
                    }
                    Err(e) => {
                        error!("Response error: {:?}", e);
                        // remove pending request
                        self.pending_requests.remove(&id);
                    }
                }
            }

            IpcMessage::DPCList(cfg) => {
                debug!("Got DPC list");
                self.model.borrow_mut().set_dpc_list(cfg);
            }
            IpcMessage::NetworkStatus(cfg) => {
                debug!("Got Network status");
                self.model.borrow_mut().update_network_status(cfg);
            }
            IpcMessage::AppStatus(app) => {
                debug!("Got AppStatus");
                self.model.borrow_mut().update_app_status(app);
            }

            IpcMessage::DownloaderStatus(dnl) => {
                debug!("Got DownloaderStatus");
                self.model.borrow_mut().update_downloader_status(dnl);
            }

            // this event is guaranteed to be sent before periodic events
            IpcMessage::AppSummary(summary) => {
                debug!("Got AppSummary");
                self.model.borrow_mut().update_app_summary(summary);
            }

            // this event is guaranteed to be sent before periodic events
            IpcMessage::NodeStatus(node_status) => {
                debug!("Got NodeStatus");
                self.model.borrow_mut().update_node_status(node_status);
            }

            IpcMessage::OnboardingStatus(o_status) => {
                debug!("Got OnboardingStatus");
                self.model.borrow_mut().update_onboarding_status(o_status);
            }

            IpcMessage::VaultStatus(status) => {
                debug!("Got VaultStatus");
                self.model.borrow_mut().update_vault_status(status);
            }

            IpcMessage::LedBlinkCounter(_led) => {
                debug!("Got LedBlinkCounter");
            }

            // this event is guaranteed to be sent before periodic events
            IpcMessage::AppsList(app_list) => {
                debug!("Got AppsList");
                self.model.borrow_mut().update_app_list(app_list);
            }

            IpcMessage::ZedAgentStatus(status) => {
                debug!("Got ZedAgentStatus");
                self.model.borrow_mut().update_zed_agent_status(status);
            }

            IpcMessage::TUIConfig(cfg) => {
                info!("== Configuration changed: TUIConfig ==");
                // update log level
                LevelFilter::from_str(&cfg.log_level).map_or_else(
                    |e| {
                        warn!("Invalid log level: {}", e);
                    },
                    |log_level| {
                        log::set_max_level(log_level);
                        info!("Log level set to: {:?}", log::max_level());
                    },
                );
                // update application config
                self.config.log_level = cfg.log_level;
                match self.config.save() {
                    Ok(_) => {
                        info!("Application Configuration saved");
                    }
                    Err(e) => {
                        error!("Failed to save configuration: {}", e);
                    }
                }
            }

            IpcMessage::TpmLogs(logs) => {
                debug!("Got TpmLogs");
                self.model.borrow_mut().update_tpm_logs(logs);
            }

            _ => {
                warn!("Unhandled IPC message: {:?}", msg);
            }
        }
    }

    pub fn send_dpc(&mut self, old: InterfaceState, new: InterfaceState) {
        let current_dpc = self.model.borrow().get_current_dpc().cloned();
        if let Some(current_dpc) = current_dpc {
            info!("send_dpc: Sending DPC for iface {}", &new.iface_name);
            let mut new_dpc = current_dpc.to_new_dpc_with_key("manual");
            // there are 3 cases:
            // 1. iface is switched DHCP -> Static
            // 2. iface is switched Static -> DHCP
            // 3. iface is switched Static -> Static with different IP
            //
            match (old.is_dhcp(), new.is_dhcp()) {
                (false, true) => {
                    // case 2
                    new_dpc
                        .get_port_by_name_mut(&new.iface_name)
                        .unwrap()
                        .to_dhcp();
                }
                (_, false) => {
                    let ip: IpAddr = new.ipv4.parse().unwrap();
                    let mask: IpAddr = new.mask.parse().unwrap();

                    // parse DNS server string and convert to Option<Vec<IpAddr>>
                    let dns_servers = new
                        .dns
                        .split(',')
                        .map(|s| s.parse::<IpAddr>().ok())
                        .flatten()
                        .collect::<Vec<IpAddr>>();

                    // same for NTP. NTP can now be either IP or FQDN
                    let ntp_servers = new
                        .ntp
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();

                    // case 1,3
                    new_dpc
                        .get_port_by_name_mut(&new.iface_name)
                        .unwrap()
                        .to_static(
                            IpNet::with_netmask(ip, mask).unwrap(),
                            new.gw.parse().unwrap(),
                            new.domain,
                            if ntp_servers.is_empty() {
                                None
                            } else {
                                Some(ntp_servers)
                            },
                            if dns_servers.is_empty() {
                                None
                            } else {
                                Some(dns_servers)
                            },
                        );
                }
                (true, true) => {
                    // this may actually happen if we add support for DHCP with some static fields e.g. custom DNS
                    // log an error for now
                    error!(
                        "send_dpc: DHCP -> DHCP transition with static fields is not supported yet but seems it is implemented in UI"
                    );
                } // do nothing
            }
            self.send_ipc_message(IpcMessage::new_request(Request::SetDPC(new_dpc)), |_| {});
        }
    }

    fn create_kmsg_task(
        &mut self,
    ) -> (
        JoinHandle<Result<()>>,
        CancellationToken,
        UnboundedReceiver<rmesg::entry::Entry>,
    ) {
        let cancel_token = CancellationToken::new();
        let cancel_token_child = cancel_token.clone();
        let (dmesg_tx, dmesg_rx) = mpsc::unbounded_channel::<rmesg::entry::Entry>();
        let is_desktop = Application::is_desktop();

        let kmsg_task: JoinHandle<Result<()>> = tokio::spawn(async move {
            if is_desktop {
                let mut index = 0;
                while !cancel_token_child.is_cancelled() {
                    let dummy_entry = rmesg::entry::Entry {
                        level: Some(rmesg::entry::LogLevel::Info),
                        message: format!("[INFO] {} Desktop mode: no kmsg", index),
                        facility: None,
                        sequence_num: None,
                        timestamp_from_system_start: None,
                    };

                    index += 1;

                    tokio::select! {
                        _ = cancel_token_child.cancelled() => {
                            info!("Kmsg task was cancelled");
                            return Ok(());
                        }
                        _ = tokio::time::timeout(tokio::time::Duration::from_millis(200), cancel_token_child.cancelled() ) => {
                            dmesg_tx.send(dummy_entry.clone()).unwrap();
                        }
                    }
                }
            } else {
                //FIXME: this stream is buggy!!! it leaves a thread behind and tokio cannot gracefully shutdown
                let mut st = rmesg::logs_stream(rmesg::Backend::Default, true, false).await?;

                while !cancel_token_child.is_cancelled() {
                    tokio::select! {
                        _ = cancel_token_child.cancelled() => {
                            info!("Kmsg task was cancelled");
                            return Ok(());
                        }
                        log = st.next() => {
                            trace!("Got log entry");
                            match log {
                                Some(Ok(log)) => {
                                    dmesg_tx.send(log).unwrap();
                                }
                                Some(Err(e)) => {
                                    warn!("Error reading kmsg: {:?}", e);
                                }
                                None => {
                                    warn!("Kmsg stream ended");
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            info!("Kmsg stream ended");
            Ok(())
        });

        (kmsg_task, cancel_token, dmesg_rx)
    }

    fn create_timer_task(
        &self,
        period: u64,
    ) -> (JoinHandle<()>, CancellationToken, UnboundedReceiver<Event>) {
        let (timer_tx, timer_rx) = mpsc::unbounded_channel::<Event>();
        let cancellation_token = CancellationToken::new();
        let cancellation_token_child = cancellation_token.clone();
        let timer_task = tokio::spawn(async move {
            while !cancellation_token_child.is_cancelled() {
                tokio::select! {
                    _ = tokio::time::timeout(tokio::time::Duration::from_millis(period), cancellation_token_child.cancelled() ) => {
                        timer_tx.send(Event::Tick).unwrap();
                    }
                }
            }
        });

        (timer_task, cancellation_token, timer_rx)
    }

    fn create_ipc_task(
        &mut self,
    ) -> (
        JoinHandle<()>,
        CancellationToken,
        UnboundedReceiver<IpcMessage>,
    ) {
        let (ipc_tx, ipc_rx) = mpsc::unbounded_channel::<IpcMessage>();
        let (ipc_cmd_tx, mut ipc_cmd_rx) = mpsc::unbounded_channel::<IpcMessage>();
        let ipc_cancel_token = CancellationToken::new();
        let ipc_cancel_token_clone = ipc_cancel_token.clone();
        self.ipc_tx = Some(ipc_cmd_tx);

        let ipc_task = tokio::spawn(async move {
            ipc_tx.send(IpcMessage::Connecting).unwrap();

            let socket_path = Application::get_socket_path();

            info!("Connecting to IPC socket {} ", &socket_path);
            let stream = IpcClient::connect(&socket_path).await.unwrap();
            let (mut sink, mut stream) = stream.split();

            ipc_tx.send(IpcMessage::Ready).unwrap();

            while !ipc_cancel_token_clone.is_cancelled() {
                let ipc_event = stream.next().fuse();

                tokio::select! {
                    _ = ipc_cancel_token_clone.cancelled() => {
                        info!("IPC task was cancelled");
                        return;
                    }
                    msg = ipc_cmd_rx.recv() => {
                        match msg {
                            Some(msg) => {
                                sink.send(msg.into()).await.unwrap();
                            }
                            None => {
                                warn!("IPC message stream ended");
                                break;
                            }
                        }
                    },
                    msg = ipc_event => {
                        match msg {
                            Some(Ok(msg)) => {
                                ipc_tx.send(IpcMessage::from(msg)).unwrap();
                            }
                            Some(Err(e)) => {
                                warn!("Error reading IPC message: {:?}", e);
                            }
                            None => {
                                warn!("IPC message stream ended");
                                break;
                            }
                        }
                    }
                }
            }
        });

        (ipc_task, ipc_cancel_token, ipc_rx)
    }

    fn create_terminal_task(&mut self) -> (JoinHandle<()>, CancellationToken) {
        let mut terminal_event_stream = TerminalWrapper::get_stream();
        let terminal_tx_clone = self.terminal_tx.clone();
        let terminal_cancel_token = CancellationToken::new();
        let terminal_cancel_token_child = terminal_cancel_token.clone();
        let terminal_task = tokio::spawn(async move {
            loop {
                let terminal_event = terminal_event_stream.next().fuse();

                tokio::select! {
                    _ = terminal_cancel_token_child.cancelled() => {
                        info!("Terminal task was cancelled");
                        return;
                    }
                    event = terminal_event => {
                        match event {
                            Some(Ok(crossterm::event::Event::Key(key))) => {
                                terminal_tx_clone.send(Event::Key(key)).unwrap();
                            }
                            Some(Ok(crossterm::event::Event::Resize(w, h))) => {
                                terminal_tx_clone.send(Event::TerminalResize(w,h)).unwrap();
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                warn!("Error reading terminal event: {:?}", e);
                            }
                            None => {
                                warn!("Terminal event stream ended");
                                break;
                            }
                        }
                    },
                }
            }
        });
        (terminal_task, terminal_cancel_token)
    }

    pub async fn run(&mut self) -> Result<()> {
        let (ipc_task, ipc_cancellation_token, mut ipc_rx) = self.create_ipc_task();

        // TODO: handle suspend/resume for the case when we give away /dev/tty
        // because we passed through the GPU to a guest VM
        let (terminal_task, terminal_cancel_token) = self.create_terminal_task();

        // spawn a timer to send tick events
        let (timer_task, timer_cancellation_token, mut timer_rx) = self.create_timer_task(500);

        // start a task to fetch kernel messages using rmesg
        let (kmsg_task, kmsg_cancellation_token, mut dmesg_rx) = self.create_kmsg_task();

        // send initial redraw event
        self.invalidate();

        #[allow(unused_assignments)]
        let mut do_redraw = true;
        let app_cancel_token = CancellationToken::new();

        // listen on the action channel and terminal channel
        while !app_cancel_token.is_cancelled() {
            // TODO: set to true by default to make life easier for now
            // Set to false in an action handler if it occurs often and doesn't require a redraw
            do_redraw = true;

            tokio::select! {
                _ = app_cancel_token.cancelled() => {
                    info!("Application cancelled");
                    break;
                }
                tick = timer_rx.recv() => {
                    match tick {
                        Some(event) => {
                            let action = self.ui.handle_event(event);
                            if let Some(action) = action {
                                trace!("Event loop got action on tick: {:?}", action);
                            }
                        }
                        None => {
                            warn!("Timer stream ended");
                            break;
                        }
                    }
                }
                event = self.terminal_rx.recv() => {
                    match event {
                        Some(Event::Key(key)) => {
                            let action = self.ui.handle_event(Event::Key(key));
                            if let Some(action) = action {
                                info!("Event loop got action: {:?}", action);

                                self.handle_action(action);
                            }
                         }
                        Some(Event::TerminalResize(w, h)) => {
                            info!("Terminal resized: {}x{}", w, h);
                        }
                        None => {
                            warn!("Terminal event stream ended");
                            break;
                        }
                        _ => {}
                    }

                }
                ipc_event = ipc_rx.recv() => {
                    match ipc_event {
                        Some(msg) => {
                            // handle IPC message
                            info!("IPC message: {:?}", msg);
                            self.handle_ipc_message(msg);
                        }
                        None => {
                            warn!("IPC message stream ended");
                            break;
                        }
                    }
                }
                action = self.action_rx.recv() => {
                    match action {
                        Some(action) => {
                            info!("Async Action: {:?}", action);
                            match action.action {
                                UiActions::Quit => {
                                    app_cancel_token.cancel();
                                }
                                _ => {}
                            }
                        }
                        None => {
                            warn!("Action stream ended");
                            break;
                        }
                    }
                }
                dmesg = dmesg_rx.recv() => {
                    match dmesg {
                        Some(entry) => {
                            // fetch all entries from the stream
                            self.model.borrow_mut().dmesg.push(entry);
                            while let Ok(entry) = dmesg_rx.try_recv() {
                                self.model.borrow_mut().dmesg.push(entry);
                            }
                        }
                        None => {
                            warn!("Dmesg stream ended");
                            break;
                        }
                    }
                }

            }
            if do_redraw {
                trace!("Redraw requested");
                self.draw_ui(self.model.clone())?;
            }
        }
        info!("Cancelling tasks");
        timer_cancellation_token.cancel();
        kmsg_cancellation_token.cancel();
        terminal_cancel_token.cancel();
        ipc_cancellation_token.cancel();
        info!("Waiting for tasks to finish");
        let _ = kmsg_task.await;
        info!("Kmsg task ended");
        terminal_task.await?;
        info!("Terminal task ended");
        //TODO: rewrite the task so we can cancel it
        ipc_task.abort();
        _ = ipc_task.await;
        info!("IPC task ended");
        timer_task.await?;
        info!("Timer task ended");
        info!("run() ended");

        Ok(())
    }

    fn invalidate(&mut self) {
        self.action_tx
            .send(Action::new("app", UiActions::Redraw))
            .unwrap();
    }

    fn draw_ui(&mut self, model: Rc<Model>) -> Result<()> {
        self.ui.draw(model);
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action.action {
            UiActions::EditIfaceConfig(iface) => {
                // get interface info by name
                let iface_data = self
                    .model
                    .borrow()
                    .network
                    .iter()
                    .find(|e| e.name == iface)
                    .cloned();
                if let Some(iface_data) = iface_data {
                    self.ui.show_ip_dialog(iface_data);
                }
            }
            UiActions::ChangeServer => {
                if self.model.borrow().node_status.is_onboarded() {
                    self.ui.message_box(
                        "WARNING",
                        "The node is onboarded and the server URL cannot be changed.",
                    );
                } else {
                    let url = self
                        .model
                        .borrow()
                        .node_status
                        .server
                        .clone()
                        .unwrap_or_default();
                    self.ui.show_server_url_dialog(&url);
                }
            }
            UiActions::AppAction(app_action) => match app_action {
                MonActions::NetworkInterfaceUpdated(old, new) => {
                    debug!("Setting DPC for {}", &old.iface_name);
                    debug!("OLD DPC: {:#?}", &old);
                    debug!("NEW DPC: {:#?}", &new);
                    if old == new {
                        debug!("Not changed, not sending DPC");
                    } else {
                        self.send_dpc(old, new);
                    }
                    self.ui.pop_layer();
                }
                MonActions::ServerUpdated(url) => {
                    debug!("Setting server URL to: {}", &url);
                    self.send_ipc_message(
                        IpcMessage::new_request(Request::SetServer(url.clone())),
                        move |app| {
                            app.model.borrow_mut().node_status.server = Some(url.clone());
                        },
                    );
                    self.ui.pop_layer();
                }
            },
            _ => {}
        }
    }
}
