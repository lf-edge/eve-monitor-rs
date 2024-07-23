use crate::device::dmesg::DmesgViewer;
use crate::events::Event;
use crate::ipc::eve_types::{
    DeviceNetworkStatus, DevicePortConfig, DevicePortConfigList, NetworkPortConfig,
    NetworkPortStatus,
};
use crate::model::Model;
use crate::raw_model::RawModel;
use crate::ui::homepage::HomePage;
use crate::ui::networkpage::create_network_page;
use crate::ui::widgets::label::LabelElement;
use crate::ui::widgets::radiogroup::RadioGroupElement;
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

use crate::actions::{IpDialogState, MainWndState, MonActions};
use crate::ipc::ipc_client::IpcClient;
use crate::ipc::message::IpcMessage;
use crate::terminal::TerminalWrapper;
use crate::ui::action::{Action, UiActions};
use crate::ui::dialog::Dialog;
use crate::ui::layer_stack::LayerStack;
use crate::ui::widgets::button::ButtonElement;
use crate::ui::widgets::input_field::InputFieldElement;
use crate::ui::window::{LayoutMap, Window};

#[derive(Debug)]
pub struct Application {
    terminal_rx: UnboundedReceiver<Event>,
    terminal_tx: UnboundedSender<Event>,
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    ipc_tx: Option<UnboundedSender<IpcMessage>>,
    ui: Ui,
    task: tokio::task::JoinHandle<()>,
}

impl Application {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
        let (terminal_tx, terminal_rx) = mpsc::unbounded_channel::<Event>();
        let terminal = TerminalWrapper::new()?;
        let mut ui = Ui::new(action_tx.clone(), terminal)?;
        ui.init();

        Ok(Self {
            terminal_rx,
            terminal_tx,
            action_rx,
            action_tx,
            ui,
            task: tokio::task::spawn(async {}),
            ipc_tx: None,
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

    pub async fn run(&mut self) -> Result<()> {
        println!("Running application");
        // we never exit the application
        // TODO: handle suspend/resume for the case when we give away /dev/tty
        let (ipc_tx, mut ipc_rx) = mpsc::unbounded_channel::<IpcMessage>();
        let (ipc_cmd_tx, mut ipc_cmd_rx) = mpsc::unbounded_channel::<IpcMessage>();
        let (timer_tx, mut timer_rx) = mpsc::unbounded_channel::<Event>();

        // to send IPC messages from the task back to app
        let ipc_tx_clone = ipc_tx.clone();

        let ipc_task = tokio::spawn(async move {
            ipc_tx_clone.send(IpcMessage::Connecting).unwrap();

            let socket_path = Application::get_socket_path();

            info!("Connecting to IPC socket {} ", &socket_path);
            let stream = IpcClient::connect(&socket_path).await.unwrap();
            let (mut sink, mut stream) = stream.split();

            ipc_tx_clone.send(IpcMessage::Ready).unwrap();

            loop {
                //let ipc_event = stream.next().fuse();

                tokio::select! {
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
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(msg)) => {
                                ipc_tx_clone.send(IpcMessage::from(msg)).unwrap();
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
        self.ipc_tx = Some(ipc_cmd_tx);

        // request data over IPC
        // self.send_ipc_message(IpcMessage::Request(Request {
        //     command: RpcCommand::Ping,
        // }));

        // we never exit the application
        // TODO: handle suspend/resume for the case when we give away /dev/tty
        // because we passed through the GPU to a guest VM
        let mut terminal_event_stream = self.ui.terminal.get_stream();

        let terminal_tx_clone = self.terminal_tx.clone();
        self.task = tokio::spawn(async move {
            loop {
                let terminal_event = terminal_event_stream.next().fuse();

                tokio::select! {
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

        // send initial redraw event
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                timer_tx.send(Event::Tick).unwrap();
            }
        });

        // send initial redraw event
        self.invalidate();

        let mut do_redraw = true;

        // listen on the action channel and terminal channel
        loop {
            // TODO: set to true by default to make life easier for now
            // Set to false in an action handler if it occurs often and doesn't require a redraw
            do_redraw = true;

            tokio::select! {
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
                            self.ui.handle_ipc_message(msg);
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
                                    break;
                                }
                                // UiActions::UserAction(MonActions::MainWndStateUpdated(state)) => {
                                _ => {}
                            }
                        }
                        None => {
                            warn!("Action stream ended");
                            break;
                        }
                    }
                }
            }
            if do_redraw {
                trace!("Redraw requested");
                self.draw_ui()?;
            }
        }

        Ok(())
    }

    fn invalidate(&mut self) {
        self.action_tx
            .send(Action::new("app", UiActions::Redraw))
            .unwrap();
    }

    fn draw_ui(&mut self) -> Result<()> {
        self.ui.draw();
        Ok(())
    }
}

struct Ui {
    terminal: TerminalWrapper,
    action_tx: UnboundedSender<Action>,
    views: Vec<LayerStack>,
    selected_tab: UiTabs,
    // this is our model :)
    model: Rc<Model>,
    raw_model: Rc<RawModel>,
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
        let model = Rc::new(Model::default());

        Ok(Self {
            terminal,
            action_tx,
            views: vec![LayerStack::new(); UiTabs::COUNT],
            selected_tab: UiTabs::default(),
            model,
            raw_model: Rc::new(RawModel::new()),
        })
    }

    pub fn create_main_wnd(&self) -> Window<MainWndState> {
        let do_layout = |w: &mut Window<MainWndState>, area: &Rect, model: &Rc<Model>| {
            let mut layout = LayoutMap::new();
            let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
            for (i, col) in cols.iter().enumerate() {
                let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
                for (j, row) in rows.iter().enumerate() {
                    let area_name = format!("{}-{}", i, j);
                    w.update_layout(area_name, *row);
                }
            }
        };

        let input = InputFieldElement::new("Input", Some("Type here")).on_char(|c: &char| {
            info!("Char: {:?}", c);
            let cap_c = c.to_uppercase().next().unwrap();
            Some(cap_c)
        });
        // .on_update(|input: &String| {
        let button = ButtonElement::new("Button");
        let rgrp = RadioGroupElement::new(vec!["Option 1", "Option 2", "Option 3"], "Radio Group");

        let clock = LabelElement::new("Clock").on_tick(|label| {
            let now = chrono::Local::now();
            let time = now.format("%H:%M:%S").to_string();
            label.set_text(time);
        });

        let wnd = Window::builder("MainWnd")
            .with_state(MainWndState {
                a: 42,
                ip: "10.208.13.5".to_string(),
            })
            .widget("3-1", Box::new(button))
            .widget("0-3", Box::new(input))
            .widget("1-1", Box::new(rgrp))
            .widget("2-2", Box::new(clock))
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
        // let dlg = Dialog::builder()
        //     .title("Dialog")
        //     .button(
        //         "Ok",
        //         Box::new(|a| {
        //             trace!("Ok button clicked");
        //         }),
        //     )
        //     .button(
        //         "Cancel",
        //         Box::new(|a| {
        //             trace!("Cancel button clicked");
        //         }),
        //     )
        //     .view(Box::new(LabelView::new("Label2", "Hello, World!")))
        //     .build();

        // self.layer_stack.push(dlg);

        let w = self.create_main_wnd();

        self.views[UiTabs::Debug as usize].push(Box::new(w));

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

        self.views[UiTabs::Debug as usize].push(Box::new(d));

        self.views[UiTabs::Home as usize].push(Box::new(HomePage::new()));

        self.views[UiTabs::Network as usize].push(Box::new(create_network_page()));

        self.views[UiTabs::Dmesg as usize].push(Box::new(DmesgViewer::new()));
    }

    fn draw(&mut self) {
        let screen_layout = Layout::vertical([Length(2), Fill(0), Length(1)]);
        let tabs_widget = Ui::tabs();

        //TODO: handle terminal event
        let _ = self.terminal.draw(|frame| {
            let area = frame.size();
            let [tabs, body, _statusbar] = screen_layout.areas(area);

            tabs_widget
                .select(self.selected_tab as usize)
                .render(tabs, frame.buffer_mut());

            // redraw from the bottom up
            let stack = &mut self.views[self.selected_tab as usize];
            let last_index = stack.len().saturating_sub(1);
            for (index, layer) in stack.iter_mut().enumerate() {
                layer.render(&body, frame, &self.model, index == last_index);
            }
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

            // handle Tab switching
            Event::Key(key)
                if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left) =>
            {
                debug!("CTRL+Left: switching tab view");
                self.selected_tab = self.selected_tab.previous();
            }
            Event::Key(key)
                if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right) =>
            {
                debug!("CTRL+Right: switching tab view");
                self.selected_tab = self.selected_tab.next();
            }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(layer) = self.views[self.selected_tab as usize].last_mut() {
                    if let Some(action) = layer.handle_event(Event::Key(key)) {
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
                }
            }
            Event::Tick => {
                // forward tick event to all layers. Callect actions
                for layer in self.views[self.selected_tab as usize].iter_mut() {
                    if let Some(action) = layer.handle_event(Event::Tick) {
                        self.action_tx.send(action).unwrap();
                    }
                }
            }
            _ => {
                debug!("Unhandled event: {:?}", event);
            }
        }

        None
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
        self.model = Rc::new(Model::from(&self.raw_model));
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
