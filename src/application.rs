use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::{thread, vec};

use anyhow::{Ok, Result};
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use log::{info, trace, warn};

use crate::dispatcher::EventDispatcher;
use crate::events::{Event, UiCommand};
use crate::mainwnd::create_main_wnd;
// use crate::events::EventCode;
use crate::terminal::TerminalWrapper;
use crate::traits::IWindow;
// use crate::ui::dialog::DialogBuilder;
// use crate::ui::input_field;
// use crate::ui::label::{self, LabelView};
// use crate::ui::statusbar::StatusBar;
// // use crate::ui::dialog::create_dialog;
// use crate::ui::window::Window;
// use crate::ui::dialog::message_box;
// use crate::ui::input_field::InputField;
// use crate::ui::statusbar::StatusBar;

#[derive(Debug)]
pub struct Application {
    //screen: Screen,
    dispatcher: EventDispatcher<Event>,
    ui: Ui,
    //timers: thread::JoinHandle<()>,
}

impl Application {
    pub fn new() -> Result<Self> {
        let dispatcher = EventDispatcher::new();
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
            // wait for an event
            let event = self.wait_for_event()?;
            match event {
                Event::Key(_) => {
                    let _ = self.ui.handle_event(event);
                }
                Event::UiCommand(cmd) => match cmd {
                    UiCommand::Redraw => {
                        let _ = self.draw_ui();
                    }
                    UiCommand::Quit => break,
                },
            }
        }
        Ok(())
    }

    fn invalidate(&mut self) {
        self.dispatcher.send(Event::UiCommand(UiCommand::Redraw));
    }

    fn wait_for_event(&self) -> Result<Event> {
        let evt = self.dispatcher.recv();
        // TODO: if evt is redraw, consume all redraw events while
        // available do not miss other events!
        evt
    }

    fn draw_ui(&mut self) -> Result<()> {
        self.ui.draw();
        Ok(())
    }
}

struct Ui {
    terminal: TerminalWrapper,
    dispatcher: EventDispatcher<Event>,
    layer_stack: Vec<Box<dyn IWindow>>,
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
            layer_stack: Vec::new(),
        })
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

        let w = create_main_wnd();

        self.layer_stack.push(Box::new(w));
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
    // fn translate_event(&self, event: EventCode) -> EventCode {
    //     match event {
    //         EventCode::Key(key) => {
    //             if key.code == crossterm::event::KeyCode::Tab
    //                 && key.kind == crossterm::event::KeyEventKind::Press
    //                 && key.modifiers.is_empty()
    //             {
    //                 EventCode::Tab
    //             } else if key.code == crossterm::event::KeyCode::BackTab
    //                 && key.kind == crossterm::event::KeyEventKind::Press
    //             {
    //                 EventCode::ShiftTab
    //             } else {
    //                 EventCode::Key(key)
    //             }
    //         }
    //         _ => event,
    //     }
    // }

    fn invalidate(&mut self) {
        self.dispatcher.send(Event::UiCommand(UiCommand::Redraw));
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        //let event = self.translate_event(event);
        info!("Ui handle_event {:?}", event);

        match event {
            // only fo debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('q')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                self.dispatcher.send(Event::UiCommand(UiCommand::Quit));
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('r')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                self.dispatcher.send(Event::UiCommand(UiCommand::Redraw));
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
                        info!("Forwarding Tab event to top layer");
                        layer.handle_key_event(key);
                    }
                }
            }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(layer) = self.layer_stack.last_mut() {
                    if let Some(evt) = layer.handle_key_event(key) {
                        match evt {
                            Event::UiCommand(cmd) => match cmd {
                                UiCommand::Redraw => {
                                    self.invalidate();
                                }
                                UiCommand::Quit => {
                                    self.dispatcher.send(Event::UiCommand(UiCommand::Quit));
                                }
                            },
                            _ => {
                                warn!("Unhandled command: {:?}", evt);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
