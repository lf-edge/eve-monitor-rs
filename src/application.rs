use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::{thread, vec};

use anyhow::{Ok, Result};
use crossterm::event::KeyModifiers;
use log::{info, trace};

use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::canvas::Label;
use ratatui::CompletedFrame;

use crate::dispatcher::EventDispatcher;
use crate::events::{Event, UiCommand};
// use crate::events::EventCode;
use crate::terminal::TerminalWrapper;
use crate::traits::{IPresenter, IWidget, IWindow};
use crate::ui::focus_tracker::{FocusMode, FocusTracker};
use crate::ui::mainwnd::MainWnd;
use crate::ui::widgets::label::LabelElement;
use crate::ui::widgets::rediogroup::{RadioGroupElement, RadioGroupState};
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
        // let label = LabelView::new("Label1", "Hello, World!");
        // let status_bar = StatusBar::new();
        // let input_field = input_field::InputField::new("Input", Some("Type here".to_string()));
        // let input1_field =
        //     input_field::InputField::new("Input1", Some("Type here too".to_string()));
        // let wnd = Window::builder()
        //     .add_view(label)
        //     .add_view(input_field)
        //     .add_view(input1_field)
        //     .add_view(status_bar)
        //     .with_layout(|frame| {
        //         let mut layout_hash = std::collections::HashMap::new();

        //         // vertical layout with 3 lines for status bar
        //         let [frame, status_bar] =
        //             Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).areas(*frame);

        //         // split the frame into two parts for the label and input field
        //         let [label, input, input1] = Layout::vertical(vec![
        //             Constraint::Length(1),
        //             Constraint::Length(3),
        //             Constraint::Length(3),
        //         ])
        //         .areas(frame);

        //         layout_hash.insert("Label1".to_string(), label);
        //         layout_hash.insert("Input".to_string(), input);
        //         layout_hash.insert("Input1".to_string(), input1);
        //         layout_hash.insert("StatusBar".to_string(), status_bar);
        //         layout_hash
        //     })
        //     .build();
        // self.layer_stack.push(wnd);

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
        let labels = vec!["a".into(), "b".into()];

        let v1 = RadioGroupElement::new(labels, "Radio Group".to_string());

        let labels = vec!["c".into(), "d".into(), "e".into()];
        let v2 = RadioGroupElement::new(labels, "Radio Group 1".to_string());

        let v3 = LabelElement::new("Label".to_string());

        let mut widgets: HashMap<String, Box<dyn IWidget>> = HashMap::new();
        widgets.insert("RadioGroup".to_string(), Box::new(v1));
        widgets.insert("RadioGroup 1".to_string(), Box::new(v2));
        widgets.insert("Label".to_string(), Box::new(v3));

        let w = MainWnd {
            ft: FocusTracker::create_from_taborder(
                vec!["RadioGroup".into(), "RadioGroup 1".into()],
                None,
                FocusMode::Wrap,
            ),
            widgets,
            layout: Default::default(),
        };
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
        trace!("Ui handle_event {:?}", event);

        match event {
            Event::Key(key) if key.code == crossterm::event::KeyCode::Char('q') => {
                self.dispatcher.send(Event::UiCommand(UiCommand::Quit));
            }
            // For debugging purposes
            Event::Key(key) if key.code == crossterm::event::KeyCode::Char('r') => {
                self.dispatcher.send(Event::UiCommand(UiCommand::Redraw));
            }
            // handle Tab
            Event::Key(key) if key.code == crossterm::event::KeyCode::Tab => {
                if let Some(layer) = self.layer_stack.last_mut() {
                    //TODO: I can hide the focus tracker from the user
                    // by making it a private field in the layer
                    // and implement handle_focus_event on the layer
                    if layer.is_focus_tracker() {
                        if key.modifiers == KeyModifiers::SHIFT {
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
                    layer.handle_key_event(key);
                }
            }
            _ => {}
        }

        Ok(())
    }
}
