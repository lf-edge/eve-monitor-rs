use std::collections::VecDeque;

use crate::events::Event;
use crate::traits::{IEventHandler, IPresenter, IWindow};
use log2::error;
use ratatui::prelude::Rect;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;
use rmesg;

#[derive(Default, Debug)]
pub struct DmesgViewer {
    mode: DmsgMode,
    buffer: VecDeque<String>,
    current_page: usize,
    max_pages: usize,
    lines_per_page: usize,
}

#[derive(Default, Debug)]
enum DmsgMode {
    #[default]
    Follow,
    Page,
}

impl DmesgViewer {
    pub fn new() -> Self {
        DmesgViewer::default()
    }
}

impl IPresenter for DmesgViewer {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        match rmesg::logs_raw(rmesg::Backend::Default, true) {
            Err(err) => error!("{}", err.to_string()),
            Ok(logs) => Paragraph::new(logs).render(*area, frame.buffer_mut()),
        };
    }
}

impl IWindow for DmesgViewer {}
impl IEventHandler for DmesgViewer {
    fn handle_event(&mut self, event: crate::events::Event) -> Option<crate::ui::action::Action> {
        match event {
            Event::Tick | Event::TerminalResize(_, _) => None, // we want this to trigger a rerender, but that will happen even if we do nothing here
            Event::Key(_) => todo!(),
        }
    }
}
