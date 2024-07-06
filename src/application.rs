use std::{thread, vec};

use anyhow::{Ok, Result};
use log::trace;

use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::canvas::Label;
use ratatui::CompletedFrame;

use crate::dispatcher::EventDispatcher;
use crate::events::EventCode;
use crate::terminal::TerminalWrapper;
use crate::ui::dialog::Dialog;
use crate::ui::label::LabelView;
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
        let wnd = Window::builder()
            .add_view(label)
            .with_layout(|frame| {
                let mut layout_hash = std::collections::HashMap::new();
                let [layout] =
                    Layout::horizontal(vec![Constraint::Percentage(10)]).areas(frame.clone());
                layout_hash.insert("Label1".to_string(), layout.clone());
                layout_hash
            })
            .build();
        self.layer_stack.push(wnd);

        let dlg = Dialog::builder()
            .title("Dialog")
            .button(
                "Ok",
                Box::new(|a| {
                    trace!("Ok button clicked");
                }),
            )
            .button(
                "Cancel",
                Box::new(|a| {
                    trace!("Cancel button clicked");
                }),
            )
            .view(Box::new(LabelView::new("Label2", "Hello, World!")))
            .build();

        self.layer_stack.push(dlg);
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
                {
                    EventCode::Tab
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
