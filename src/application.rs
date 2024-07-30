use crate::device::dmesg::{self, DmesgViewer};
use crate::events::Event;
use crate::model::{Model, MonitorModel};
use crate::raw_model::RawModel;
use crate::traits::{IEventHandler, IPresenter};
use crate::ui::homepage::HomePage;
use crate::ui::netconf::{self, NetworkDialog};
use crate::ui::networkpage::create_network_page;
use crate::ui::statusbar::{create_status_bar, StatusBarState};
use crate::ui::widgets::label::LabelElement;
use crate::ui::widgets::radiogroup::RadioGroupElement;
use crate::ui::widgets::spin_box::{SpinBoxElement, SpinBoxLayout};
use core::fmt::Debug;

use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result::Ok;

use anyhow::Result;
use log::{debug, info, trace, warn};

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Constraint::Fill;
use ratatui::prelude::Constraint::Length;
use ratatui::prelude::Stylize;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Tabs;
use ratatui::widgets::Widget;

use strum::EnumCount;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use futures::{FutureExt, SinkExt, StreamExt};

use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::actions::{IpDialogState, MainWndState, MonActions};
use crate::ipc::ipc_client::IpcClient;
use crate::ipc::message::{IpcMessage, Request};
use crate::terminal::TerminalWrapper;
use crate::ui::action::{Action, UiActions};
use crate::ui::dialog::Dialog;
use crate::ui::layer_stack::LayerStack;
use crate::ui::widgets::button::ButtonElement;
use crate::ui::widgets::input_field::{InputFieldElement, InputModifiers};
use crate::ui::window::Window;

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
        std::env::var("XDG_RUNTIME_DIR").is_ok()
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

            loop {
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
}

struct Ui {
    terminal: TerminalWrapper,
    action_tx: UnboundedSender<Action>,
    views: Vec<LayerStack>,
    selected_tab: UiTabs,
    status_bar: Window<StatusBarState>,
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
enum UiTabs {
    #[default]
    Debug,
    Home,
    Network,
    Applications,
    Dmesg,
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ui :)")
    }
}

impl Ui {
    fn new(action_tx: UnboundedSender<Action>, terminal: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            terminal,
            action_tx,
            views: vec![LayerStack::new(); UiTabs::COUNT],
            selected_tab: UiTabs::default(),
            status_bar: create_status_bar(),
        })
    }

    pub fn create_main_wnd(&self) -> Window<MainWndState> {
        let do_layout = |w: &mut Window<MainWndState>, area: &Rect, _model: &Rc<Model>| {
            let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
            for (i, col) in cols.iter().enumerate() {
                let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
                for (j, row) in rows.iter().enumerate() {
                    let area_name = format!("{}-{}", i, j);
                    w.update_layout(area_name, *row);
                }
            }
        };

        let input = InputFieldElement::new("Gateway", Some("delete me"))
            .on_char(|c: &char| {
                info!("Char: {:?}", c);

                if c.is_digit(10) || *c == '.' {
                    return Some(*c);
                }
                None
            })
            .with_modifiers(vec![
                InputModifiers::DisplayMode,
                InputModifiers::DisplayCaption,
            ])
            .with_size_hint((19, 3).into())
            .with_text_hint("192.168.0.1");
        let button = ButtonElement::new("Button");
        let radiogroup =
            RadioGroupElement::new(vec!["Option 1", "Option 2", "Option 3"], "Radio Group");

        let clock = LabelElement::new("Clock").on_tick(|label| {
            let now = chrono::Local::now();
            let time = now.format("%H:%M:%S").to_string();
            label.set_text(time);
        });

        let spinner = SpinBoxElement::new(vec!["Option 1", "Option 2", "Option 3"])
            .selected(1)
            .layout(SpinBoxLayout::Vertical)
            .size_hint(16);

        let spinner_2 = SpinBoxElement::new(vec!["DHCP", "Static", "Option 3"])
            .selected(1)
            .layout(SpinBoxLayout::Horizontal)
            .size_hint(16);

        let wnd = Window::builder("MainWnd")
            .with_state(MainWndState {
                a: 42,
                ip: "10.208.13.5".to_string(),
            })
            .widget("3-1", button)
            .widget("0-3", input)
            .widget("1-1", radiogroup)
            .widget("2-2", clock)
            .widget("3-3", spinner)
            .widget("3-2", spinner_2)
            .with_layout(do_layout)
            .with_focused_view("0-3")
            .on_action(|action, state: &mut MainWndState| {
                debug!("on_action Action: {:?}", action);
                match action.action {
                    UiActions::CheckBox { checked: _ } => todo!(),
                    UiActions::RadioGroup { selected } => {
                        info!("RadioGroup updated: {}", selected);
                    }
                    UiActions::Input { text } => {
                        info!("Input updated: {}", &text);
                        state.ip = text;
                    }
                    UiActions::ButtonClicked(_) => {
                        state.a += 1;
                        info!("Button clicked: counter {}", state.a);
                        // Send user action to indicate that the state was updated
                        return Some(UiActions::MonActions(MonActions::MainWndStateUpdated(
                            state.clone(),
                        )));
                    }
                    _ => {
                        if action.action != UiActions::Redraw {
                            warn!("Unhandled action: {:?}", action);
                        }
                    }
                }
                // match action.action {
                //     MonActions::ButtonClicked(label) => {
                //         state.a += 1;
                //         info!("Button clicked: {} counter {}", label, state.a);
                //         return Some(MonActions::MainWndStateUpdated(state.clone()));
                //     }
                //     MonActions::InputUpdated(input) => {
                //         info!("Input updated: {}", input);
                //         return Some(MonActions::MainWndStateUpdated(state.clone()));
                //     }
                //     _ => {}
                // }
                None
            })
            .build()
            .unwrap();

        wnd
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

    fn init(&mut self) {
        let w = self.create_main_wnd();

        self.views[UiTabs::Debug as usize].push(Box::new(w));

        let s = IpDialogState {
            ip: "10.208.13.10".to_string(),
            mode: "DHCP".to_string(),
            gw: "1.1.1.1".to_string(),
        };

        let d: Dialog<MonActions> = Dialog::new(
            (50, 20),
            "confirm".to_string(),
            vec!["Ok".to_string(), "Cancel".to_string()],
            "Cancel",
            MonActions::NetworkInterfaceUpdated(s),
        );

        self.views[UiTabs::Debug as usize].push(Box::new(d));

        self.views[UiTabs::Home as usize].push(Box::new(HomePage::new()));

        self.views[UiTabs::Network as usize].push(Box::new(create_network_page()));

        self.views[UiTabs::Dmesg as usize].push(Box::new(DmesgViewer::new()));
    }

    fn draw(&mut self, model: Rc<Model>) {
        let screen_layout = Layout::vertical([Length(2), Fill(0), Length(3)]);
        let tabs_widget = Ui::tabs();

        //TODO: handle terminal event
        let _ = self.terminal.draw(|frame| {
            let area = frame.size();
            let [tabs, body, statusbar_rect] = screen_layout.areas(area);

            tabs_widget
                .select(self.selected_tab as usize)
                .render(tabs, frame.buffer_mut());

            // redraw from the bottom up
            let stack = &mut self.views[self.selected_tab as usize];
            let last_index = stack.len().saturating_sub(1);
            for (index, layer) in stack.iter_mut().enumerate() {
                layer.render(&body, frame, &model, index == last_index);
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

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        debug!("Ui handle_event {:?}", event);

        match event {
            // only fo debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('q')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+q: application Quit requested");
                self.action_tx
                    .send(Action::new("user", UiActions::Quit))
                    .unwrap();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('r')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+r: manual Redraw requested");
                self.invalidate();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('p')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+p: manual layer.pop() requested");
                self.views[self.selected_tab as usize].pop();
            }

            // show dialog on ctrl+d
            Event::Key(key)
                if (key.code == KeyCode::Char('d')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+d: show dialog");

                let s = IpDialogState {
                    ip: "10.208.13.10".to_string(),
                    mode: "DHCP".to_string(),
                    gw: "1.1.1.1".to_string(),
                };

                let d: Dialog<MonActions> = Dialog::new(
                    (50, 30),
                    "confirm".to_string(),
                    vec!["Ok".to_string(), "Cancel".to_string()],
                    "Cancel",
                    MonActions::NetworkInterfaceUpdated(s),
                );
                self.views[self.selected_tab as usize].push(Box::new(d));
            }

            // show network edit dialog on ctrl+e
            Event::Key(key)
                if (key.code == KeyCode::Char('e')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+e: show dialog");

                // let s = IpDialogState {
                //     ip: "10.208.13.10".to_string(),
                //     mode: "DHCP".to_string(),
                //     gw: "1.1.1.1".to_string(),
                // };

                let d: NetworkDialog = NetworkDialog::new();
                self.views[self.selected_tab as usize].push(Box::new(d));
            }

            // handle Tab switching
            // Event::Key(key)
            //     if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left) =>
            // {
            //     debug!("CTRL+Left: switching tab view");
            //     self.selected_tab = self.selected_tab.previous();
            // }
            // Event::Key(key)
            //     if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right) =>
            // {
            //     debug!("CTRL+Right: switching tab view");
            //     self.selected_tab = self.selected_tab.next();
            // }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(action) = self.views[self.selected_tab as usize]
                    .last_mut()?
                    .handle_event(Event::Key(key))
                {
                    match action.action {
                        UiActions::DismissDialog => {
                            self.views[self.selected_tab as usize].pop();
                            // self.invalidate();
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
