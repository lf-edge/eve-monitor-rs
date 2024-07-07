use std::{thread, vec};

use anyhow::{Ok, Result};
use crossterm::event::KeyModifiers;
use log::trace;

use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::canvas::Label;
use ratatui::CompletedFrame;

use crate::dispatcher::EventDispatcher;
use crate::events::EventCode;
use crate::terminal::TerminalWrapper;
use crate::ui::dialog::Dialog;
use crate::ui::input_field;
use crate::ui::label::{self, LabelView};
use crate::ui::statusbar::StatusBar;
// use crate::ui::dialog::create_dialog;
use crate::ui::window::Window;
// use crate::ui::dialog::message_box;
// use crate::ui::input_field::InputField;
// use crate::ui::statusbar::StatusBar;

#[derive(Debug)]
pub struct Application {
    //screen: Screen,
    dispatcher: EventDispatcher<EventCode>,
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
                EventCode::Redraw => self.draw_ui()?,
                EventCode::Quit => break Ok(()),

                _ => {
                    if let Some(evt) = self.ui.handle_event(event) {
                        self.dispatcher.send(evt);
                    }
                }
            }
        }
    }

    fn invalidate(&mut self) {
        self.dispatcher.send(EventCode::Redraw)
    }

    fn wait_for_event(&self) -> Result<EventCode> {
        let evt = self.dispatcher.recv();
        //if evt is redraw, consume all redraw events while available
        // do not miss other events!
        // if let Ok(Event::Redraw) = evt {
        //     while let Some(Event::Redraw) = self.dispatcher.try_recv() {}
        // }
        evt
    }

    fn draw_ui(&mut self) -> Result<()> {
        self.ui.draw().unwrap();
        Ok(())
    }
}

#[derive(Debug)]
struct Ui {
    screen: TerminalWrapper,
    dispatcher: EventDispatcher<EventCode>,
    layer_stack: Vec<Window>,
}

impl Ui {
    fn new(dispatcher: EventDispatcher<EventCode>, screen: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            screen: screen,
            dispatcher: dispatcher,
            layer_stack: Vec::new(),
        })
    }
    fn init(&mut self) {
        let label = LabelView::new("Label1", "Hello, World!");
        let status_bar = StatusBar::new();
        let input_field = input_field::InputField::new("Input", Some("Type here".to_string()));
        let input1_field =
            input_field::InputField::new("Input1", Some("Type here too".to_string()));
        let wnd = Window::builder()
            .add_view(label)
            .add_view(input_field)
            .add_view(input1_field)
            .add_view(status_bar)
            .with_layout(|frame| {
                let mut layout_hash = std::collections::HashMap::new();

                // vertical layout with 3 lines for status bar
                let [frame, status_bar] =
                    Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).areas(*frame);

                // split the frame into two parts for the label and input field
                let [label, input, input1] = Layout::vertical(vec![
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .areas(frame);

                layout_hash.insert("Label1".to_string(), label);
                layout_hash.insert("Input".to_string(), input);
                layout_hash.insert("Input1".to_string(), input1);
                layout_hash.insert("StatusBar".to_string(), status_bar);
                layout_hash
            })
            .build();
        self.layer_stack.push(wnd);

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
    }
    fn draw(&mut self) -> Result<CompletedFrame> {
        Ok(self.screen.draw(|frame| {
            let area = frame.size();
            // redraw from the bottom up
            for layer in self.layer_stack.iter_mut() {
                layer.render(&area, frame);
            }
        })?)
    }
    fn translate_event(&self, event: EventCode) -> EventCode {
        match event {
            EventCode::Key(key) => {
                if key.code == crossterm::event::KeyCode::Tab
                    && key.kind == crossterm::event::KeyEventKind::Press
                    && key.modifiers.is_empty()
                {
                    EventCode::Tab
                } else if key.code == crossterm::event::KeyCode::BackTab
                    && key.kind == crossterm::event::KeyEventKind::Press
                {
                    EventCode::ShiftTab
                } else {
                    EventCode::Key(key)
                }
            }
            _ => event,
        }
    }
    fn handle_event(&mut self, event: EventCode) -> Option<EventCode> {
        let event = self.translate_event(event);
        trace!("Ui handle_event {:?}", event);

        match event {
            EventCode::Key(key) => {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    return Some(EventCode::Quit);
                }
            }
            _ => {}
        }

        if let Some(layer) = self.layer_stack.last_mut() {
            return layer.handle_event(&event);
        }
        None
    }
}
