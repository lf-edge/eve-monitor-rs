use crate::events::Event;
use crate::traits::IWidgetPresenter;
use crate::ui::homepage::HomePage;
use core::fmt::Debug;

use std::result::Result::Ok;

use anyhow::Result;
use log::{debug, info, warn};

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
use ratatui::Frame;

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
use crate::ipc::message::{IpcMessage, Request, RpcCommand};
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

    pub async fn run(&mut self) -> Result<()> {
        println!("Running application");
        // we never exit the application
        // TODO: handle suspend/resume for the case when we give away /dev/tty
        let (ipc_tx, mut ipc_rx) = mpsc::unbounded_channel::<IpcMessage>();
        let (ipc_cmd_tx, mut ipc_cmd_rx) = mpsc::unbounded_channel::<IpcMessage>();

        // to send IPC messages from the task back to app
        let ipc_tx_clone = ipc_tx.clone();

        let ipc_task = tokio::spawn(async move {
            ipc_tx_clone.send(IpcMessage::Connecting).unwrap();

            info!("Connecting to IPC socket");
            let stream = IpcClient::connect("/tmp/monitor.sock").await.unwrap();
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
        self.send_ipc_message(IpcMessage::Request(Request {
            command: RpcCommand::Ping,
        }));

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
        self.invalidate();

        // listen on the action channel and terminal channel
        loop {
            tokio::select! {
                event = self.terminal_rx.recv() => {
                    match event {
                        Some(Event::Key(key)) => {
                            let action = self.ui.handle_event(Event::Key(key));
                            if let Some(action) = action {
                                info!("Event loop got action: {:?}", action);
                            }
                                self.draw_ui().unwrap();
                            }
                        None => {
                            warn!("Terminal event stream ended");
                            break;
                        }
                    }

                }
                ipc_event = ipc_rx.recv() => {
                    match ipc_event {
                        Some(msg) => {
                            // handle IPC message
                            info!("IPC message: {:?}", msg);
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
                                UiActions::Redraw => {
                                    self.draw_ui().unwrap();
                                }
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
    _a: u32,
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
enum UiTabs {
    #[default]
    Debug,
    Home,
    Network,
    Applications,
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
            _a: 0,
        })
    }

    pub fn create_main_wnd(&self) -> Window<MainWndState> {
        let do_layout = |area: &Rect| -> Option<LayoutMap> {
            let mut layout = LayoutMap::new();
            let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
            for (i, col) in cols.iter().enumerate() {
                let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
                for (j, row) in rows.iter().enumerate() {
                    let area_name = format!("{}-{}", i, j);
                    layout.insert(area_name, *row);
                }
            }
            Some(layout)
        };

        let input = InputFieldElement::new("Input", Some("Type here")).on_char(|c: &char| {
            info!("Char: {:?}", c);
            let cap_c = c.to_uppercase().next().unwrap();
            Some(cap_c)
        });
        // .on_update(|input: &String| {
        let button = ButtonElement::new("Button");

        let wnd = Window::builder("MainWnd")
            .with_state(MainWndState {
                a: 42,
                ip: "10.208.13.5".to_string(),
            })
            .widget("3-1", Box::new(button))
            .widget("0-3", Box::new(input))
            .with_layout(do_layout)
            .with_focused_view("Input")
            .on_action(|action, state: &mut MainWndState| {
                debug!("on_action Action: {:?}", action);
                match action.action {
                    UiActions::CheckBox { checked: _ } => todo!(),
                    UiActions::RadioGroup { selected: _ } => todo!(),
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
                        warn!("Unhandled action: {:?}", action);
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
                layer.render(&body, frame, index == last_index);
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
            // handle Tab key
            Event::Key(key) if (key.code == KeyCode::Tab || key.code == KeyCode::BackTab) => {
                if let Some(layer) = self.views[self.selected_tab as usize].last_mut() {
                    let action = layer.handle_event(Event::Key(key));
                    if let Some(action) = action {
                        return Some(action);
                    }
                }
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
