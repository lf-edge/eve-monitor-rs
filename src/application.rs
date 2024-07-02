use anyhow::Result;
use ratatui::Frame;
use ratatui::{buffer::Buffer, widgets::WidgetRef};

use ratatui::CompletedFrame;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};
use Constraint::{Length, Min};

use crate::dispatcher::EventDispatcher;
use crate::events::Event;
use crate::terminal::TerminalWrapper;
use crate::traits::Component;
use crate::ui::dialog::message_box;
use crate::ui::input_field::InputField;
use crate::ui::statusbar::StatusBar;

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
        //     dispatcher_clone.send(Event::Redraw);
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
                Event::Redraw => self.draw_ui()?,

                _ => {
                    if let Some(evt) = self.ui.handle_event(event) {
                        self.dispatcher.send(evt);
                    }
                }
            }
        }
    }

    fn invalidate(&mut self) {
        self.dispatcher.send(Event::Redraw)
    }

    fn wait_for_event(&self) -> Result<Event> {
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

struct MainWndWidget<'a> {
    status_bar: StatusBar<'a>,
    //input_field: InputField,
}

impl<'a> MainWndWidget<'a> {
    fn new() -> Self {
        Self {
            status_bar: StatusBar::new(),
            // input_field: InputField {
            //     widget: InputFieldWidget {},
            //     state: InputFieldState {
            //         caption: "Input".to_string(),
            //         value: Some("initial".to_string()),
            //         input_position: 0,
            //         cursor_position: Position::new(0, 0),
            //     },
            // },
        }
    }
    fn calculate_layout<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let layout = Layout::vertical([Min(0), Length(3)]).areas(area);
        layout
    }
}

impl<'a> Widget for MainWndWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf)
    }
}

// impl<'a> Deref for MainWndWidget<'a> {
//     type Target = MainWndWidget<'a>;

//     fn deref(&self) -> &Self::Target {
//         unsafe { std::mem::transmute(self) }
//     }
// }

// impl DerefMut for MainWndWidget<'_> {
//     fn deref_mut(&mut self) -> &mut Self {
//         self
//     }
// }
impl<'a> WidgetRef for MainWndWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

impl<'a> WidgetRef for &MainWndWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_inner(area, buf);
    }
}

impl<'a> MainWndWidget<'a> {
    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let [content, status] = self.calculate_layout(area);
        self.status_bar.render_ref(status, buf);
        let input_layout = Layout::vertical([Length(3), Min(0)]).split(content)[0];
        //self.input_field.render(&input_layout, buf);
    }
}

struct MainWnd<'a> {
    widget: MainWndWidget<'a>,
}

impl<'a> MainWnd<'a> {
    fn new() -> Self {
        Self {
            widget: MainWndWidget::new(),
        }
    }
}

impl<'a> Component for MainWnd<'a> {
    // fn calculate_layout(&mut self, area: &Rect) {
    //     let layout = self.calculate_layout(area);
    //     //self.status_bar.border.calculate_layout(&layout[1]);
    // }
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        frame.render_widget_ref(&self.widget, *area);
    }
    fn handle_event(&mut self, event: &Event) -> Option<Event> {
        match event {
            Event::Key(key) => {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    // exit the application
                    std::process::exit(0);
                } else if key.code == crossterm::event::KeyCode::Char('d') {
                    // create modal dialog
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

struct FocusState {
    focused: bool,
}

#[derive(Debug)]
struct Ui {
    screen: TerminalWrapper,
    dispatcher: EventDispatcher<Event>,
    layer_stack: Vec<Box<dyn Component>>,
}

impl Ui {
    fn new(dispatcher: EventDispatcher<Event>, screen: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            screen: screen,
            dispatcher: dispatcher,
            layer_stack: Vec::new(),
        })
    }
    fn init(&mut self) {
        let main = Box::new(MainWnd::new());
        self.layer_stack.push(main);
        self.layer_stack.push(Box::new(InputField::new(
            "caption".to_string(),
            Some("value".to_string()),
        )));
        let dlg = message_box(
            "title",
            "my message",
            vec!["OK".to_string(), "Cancel".to_string()],
        );
        self.layer_stack.push(Box::new(dlg));
    }
    fn draw(&mut self) -> Result<CompletedFrame> {
        Ok(self.screen.draw(|frame| {
            let area = frame.size();
            if let Some(layer) = self.layer_stack.last_mut() {
                layer.render(&area, frame);
            }
        })?)
    }
    fn translate_event(&self, event: Event) -> Event {
        match event {
            Event::Key(key) => {
                if key.code == crossterm::event::KeyCode::Tab {
                    Event::Tab
                } else {
                    Event::Key(key)
                }
            }
            _ => event,
        }
    }
    fn handle_event(&mut self, event: Event) -> Option<Event> {
        // convert Tab key to Tab event
        let event = self.translate_event(event);

        if let Some(layer) = self.layer_stack.last_mut() {
            return layer.handle_event(&event);
        }
        None
    }
}
