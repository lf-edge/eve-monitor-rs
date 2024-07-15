use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::time::Duration;
use std::{thread, vec};

use anyhow::{Ok, Result};
use crossbeam::select;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use log::{debug, info, trace, warn};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::actions::{IpDialogState, MainWndState, MonActions};
use crate::dispatcher::EventDispatcher;
use crate::events::{Event, UiCommand};
use crate::terminal::TerminalWrapper;
use crate::traits::{IAction, IWindow};
use crate::ui::action::{Action, UiActions};
use crate::ui::dialog::Dialog;
use crate::ui::layer_stack::LayerStack;
use crate::ui::widgets::button::ButtonElement;
use crate::ui::widgets::input_field::InputFieldElement;
use crate::ui::window::{LayoutMap, WidgetMap, Window};

#[derive(Debug)]
pub struct Application {
    dispatcher: EventDispatcher<Event>,
    action_handler: EventDispatcher<Action<MonActions>>,
    ui: Ui,
    //timers: thread::JoinHandle<()>,
}

impl EventDispatcher<Event> {
    pub fn send_ui_command(&self, cmd: UiCommand) {
        self.send(Event::UiCommand(cmd));
    }
    pub fn send_redraw(&self) {
        self.send_ui_command(UiCommand::Redraw);
    }
}

impl Application {
    pub fn new() -> Result<Self> {
        let dispatcher = EventDispatcher::new();
        let action_handler = EventDispatcher::new();
        let terminal = TerminalWrapper::new(dispatcher.clone())?;
        let mut ui = Ui::new(dispatcher.clone(), terminal)?;
        ui.init();

        let dispatcher_clone = dispatcher.clone();
        // let timers = thread::spawn(move || loop {
        //     thread::sleep(std::time::Duration::from_millis(50));
        //     dispatcher_clone.send(EventCode::Redraw);
        // });
        Ok(Self {
            dispatcher,
            action_handler,
            ui,
            //timers,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Running application");
        // we never exit the application
        // TODO: handle suspend/resume for the case when we give away /dev/tty
        // because we passed through the GPU to a guest VM

        //send inital redraw event
        self.invalidate();

        loop {
            select! {
                recv(self.dispatcher) -> event => {
                    let event = event?;
                    match event {
                        Event::Key(_)  => {
                            // handle action immediately to response to user input
                            let action = self.ui.handle_event(event);
                            if let Some(action) = action {
                                info!("Event loop got action: {:?}", action);
                                self.draw_ui()?;
                            }
                        }
                        Event::UiCommand(cmd) => match cmd {
                            UiCommand::Redraw => {
                                // TODO: if evt is redraw, consume all redraw events
                                // to minimize the number of redraws
                                // TODO 2: we could implement partial screen update
                                let _ = self.draw_ui();
                            }
                            UiCommand::Quit => break,
                        },
                    }
                }
                recv(self.action_handler) -> action => {
                    // these are external actions produced by async tasks
                    let action = action?;
                    info!("Async Action: {:?}", action);
                }
                // default(Duration::from_millis(500)) => {
                //     // to emulate the timer
                //     // TODO: add subscription for actions so we can do partial screen updates
                //     //self.invalidate();
                //     self.ui.draw();
                // }
            }
        }
        Ok(())
    }

    fn invalidate(&mut self) {
        self.dispatcher.send_redraw();
    }

    fn draw_ui(&mut self) -> Result<()> {
        self.ui.draw();
        Ok(())
    }
}

struct Ui {
    terminal: TerminalWrapper,
    dispatcher: EventDispatcher<Event>,
    layer_stack: LayerStack<MonActions>,
    // this is our model :)
    a: u32,
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ui :)")
    }
}

impl Ui {
    fn new(dispatcher: EventDispatcher<Event>, terminal: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            terminal,
            dispatcher,
            layer_stack: LayerStack::new(),
            a: 0,
        })
    }
    pub fn create_main_wnd(&self) -> Window<MonActions, MainWndState> {
        let do_layout = |area: &Rect, layout: &mut LayoutMap| {
            let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
            for (i, col) in cols.iter().enumerate() {
                let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
                for (j, row) in rows.iter().enumerate() {
                    let area_name = format!("{}-{}", i, j);
                    layout.insert(area_name, *row);
                }
            }
            Ok(())
        };

        let do_render = Box::new(
            |_area: &Rect,
             frame: &mut Frame<'_>,
             layout: &LayoutMap,
             widgets: &mut WidgetMap<MonActions>| {
                // let r = layout.get("0-0").unwrap();
                // let rg = widgets.get_mut("RadioGroup").unwrap();
                // rg.render(r, frame);

                // let r = layout.get("0-1").unwrap();
                // let rg = widgets.get_mut("RadioGroup 1").unwrap();
                // rg.render(r, frame);

                // let r = layout.get("3-3").unwrap();
                // let rg = widgets.get_mut("Label").unwrap();
                // rg.render(r, frame);

                let r = layout.get("3-0").unwrap();
                let rg = widgets.get_mut("Input").unwrap();
                rg.render(r, frame);

                let r = layout.get("0-2").unwrap();
                let rg = widgets.get_mut("Button").unwrap();
                rg.render(r, frame);
            },
        );

        // let rg1 = Box::new(RadioGroupElement::new(
        //     vec!["Option 1", "Option 2"],
        //     "Radio Group",
        // ));

        // let rg2 = Box::new(RadioGroupElement::new(
        //     vec!["Option 1", "Option 2"],
        //     "Radio Group 1",
        // ));

        //let label = LabelElement::new("Label");

        let input = InputFieldElement::new("Input", Some("Type here")).on_char(|c: &char| {
            info!("Char: {:?}", c);
            let cap_c = c.to_uppercase().next().unwrap();
            Some(cap_c)
        });
        // .on_update(|input: &String| {
        //     info!("Input updated: {}", input);
        //     Some(MonActions::InputUpdated(input.clone()))
        // });

        let on_click = Box::new(|label: &String| -> Option<UiActions<MonActions>> {
            info!("Button clicked {}", label);
            Some(UiActions::ButtonClicked(label.clone()))
        });

        let button = ButtonElement::<MonActions>::new("Button").on_click(on_click);

        let wnd = Window::builder("MainWnd")
            .with_state(MainWndState {
                a: 42,
                ip: "10.208.13.5".to_string(),
            })
            .widget("Button", Box::new(button))
            .widget("Input", Box::new(input))
            .with_layout(do_layout)
            .with_render(do_render)
            .with_focused_view("Input")
            .on_action(|action, state: &mut MainWndState| {
                debug!("on_action Action: {:?}", action);
                match action.action {
                    UiActions::CheckBox { checked } => todo!(),
                    UiActions::RadioGroup { selected } => todo!(),
                    UiActions::Input { text } => {
                        info!("Input updated: {}", &text);
                        state.ip = text;
                    }
                    UiActions::ButtonClicked(_) => {
                        state.a += 1;
                        info!("Button clicked: counter {}", state.a);
                        // Send user action to indicate that the state was updated
                        return Some(UiActions::new_user_action(MonActions::MainWndStateUpdated(
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

        self.layer_stack.push(Box::new(w));

        let s = IpDialogState {
            ip: "10.208.13.10".to_string(),
            mode: "DHCP".to_string(),
            gw: "1.1.1.1".to_string(),
        };

        let d: Dialog<MonActions, IpDialogState> = Dialog::new(
            (50, 30),
            vec!["Ok".to_string(), "Cancel".to_string()],
            "Cancel",
            s,
        );

        self.layer_stack.push(Box::new(d));
    }
    fn draw(&mut self) {
        //TODO: handle terminal event
        let _ = self.terminal.draw(|frame| {
            let area = frame.size();
            // redraw from the bottom up
            for layer in self.layer_stack.iter_mut() {
                layer.do_layout(&area);
                layer.render(&area, frame);
            }
        });
    }

    fn invalidate(&mut self) {
        self.dispatcher.send(Event::UiCommand(UiCommand::Redraw));
    }

    fn handle_event(&mut self, event: Event) -> Option<Action<MonActions>> {
        debug!("Ui handle_event {:?}", event);

        match event {
            // only fo debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('q')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+q: application Quit requested");
                self.dispatcher.send(Event::UiCommand(UiCommand::Quit));
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('r')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+r: manual Redraw requested");
                self.dispatcher.send_redraw();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('p')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+p: manual layer.pop() requested");
                self.layer_stack.pop();
                self.dispatcher.send_redraw();
            }
            // handle Tab
            Event::Key(key) if (key.code == KeyCode::Tab || key.code == KeyCode::BackTab) => {
                if let Some(layer) = self.layer_stack.last_mut() {
                    //TODO: I can hide the focus tracker from the user
                    // by making it a private field in the layer
                    // and implement handle_focus_event on the layer
                    if layer.is_focus_tracker() {
                        if key.code == KeyCode::Tab {
                            layer.focus_prev();
                        } else {
                            layer.focus_next();
                        }
                        self.invalidate();
                    } else {
                        // forward the event to the top layer
                        debug!("Forwarding Tab event to top layer");
                        let action = layer.handle_key_event(key);
                        if let Some(action) = action {
                            return Some(action);
                        }
                    }
                }
            }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(layer) = self.layer_stack.last_mut() {
                    if let Some(action) = layer.handle_key_event(key) {
                        match action.action {
                            UiActions::DismissDialog => {
                                self.layer_stack.pop();
                                self.invalidate();
                            }
                            _ => {
                                return Some(action);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }
}
