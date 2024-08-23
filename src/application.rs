use crate::events::Event;
use crate::model::{Model, MonitorModel};
use crate::raw_model::RawModel;
use crate::ui::ui::Ui;
use core::fmt::Debug;

use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result::Ok;

use anyhow::Result;
use log::{debug, info, trace, warn};

use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use futures::{FutureExt, SinkExt, StreamExt};

use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::ipc::ipc_client::IpcClient;
use crate::ipc::message::{IpcMessage, Request};
use crate::terminal::TerminalWrapper;
use crate::ui::action::{Action, UiActions};

#[derive(Debug)]
pub struct Application {
    terminal_rx: UnboundedReceiver<Event>,
    terminal_tx: UnboundedSender<Event>,
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    ipc_tx: Option<UnboundedSender<IpcMessage>>,
    ui: Ui,
    // this is our model :)
    model: Rc<Model>,
    raw_model: RawModel,
}

impl Application {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
        let (terminal_tx, terminal_rx) = mpsc::unbounded_channel::<Event>();
        let terminal = TerminalWrapper::open_terminal()?;
        let mut ui = Ui::new(action_tx.clone(), terminal)?;
        let model = Rc::new(RefCell::new(MonitorModel::default()));

        ui.init();

        Ok(Self {
            terminal_rx,
            terminal_tx,
            action_rx,
            action_tx,
            ui,
            ipc_tx: None,
            model,
            raw_model: RawModel::new(),
        })
    }

    pub fn send_ipc_message(&self, msg: IpcMessage) {
        if let Some(ipc_tx) = &self.ipc_tx {
            ipc_tx.send(msg).unwrap();
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
            IpcMessage::DPCList(cfg) => {
                debug!("Got DPC list");
                self.raw_model.set_dpc_list(cfg);
            }
            IpcMessage::NetworkStatus(cfg) => {
                debug!("Got Network status");
                self.raw_model.set_network_status(cfg);
            }
            IpcMessage::AppStatus(app) => {
                debug!("Got AppStatus");
            }

            IpcMessage::DownloaderStatus(cfg) => {
                self.raw_model.set_downloader_status(cfg);
                debug!("Got DownloaderStatus");
            }

            _ => {
                warn!("Unhandled IPC message: {:?}", msg);
            }
        }
        self.model
            .borrow_mut()
            .update_from_raw_model(&self.raw_model);
    }

    pub fn send_dpc(&self) {
        if let Some(current_dpc) = self.raw_model.get_current_dpc() {
            let mut dpc = current_dpc.clone();

            dpc.key = "manual".to_string();
            dpc.time_priority = chrono::Utc::now();

            self.send_ipc_message(IpcMessage::new_request(Request::SetDPC(dpc)));
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
                            info!("Got log entry");
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
        let mut terminal_event_stream = self.ui.terminal.get_stream();
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
        println!("Running application");

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
                            if (key.code == KeyCode::Char('s')) && (key.modifiers == KeyModifiers::CONTROL)
                            {
                                debug!("CTRL+s: sending IPC message");
                                self.send_dpc();
                            }

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
                let model = self.model.borrow();
                let iface_data = model
                    .network
                    .iter()
                    .find(|e| e.name == iface)
                    .map(|e| e.clone());
                if let Some(iface_data) = iface_data {
                    self.ui.show_ip_dialog(iface_data);
                }
            }
            _ => {}
        }
    }
}
