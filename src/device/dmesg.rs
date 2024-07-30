use crate::model::Model;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::events::Event;
use crate::traits::{IEventHandler, IPresenter, IWindow};
use log::trace;
use ratatui::prelude::Rect;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

#[derive(Default, Debug)]
pub struct DmesgViewer {
    _mode: DmsgMode,
    _buffer: VecDeque<String>,
    _current_page: usize,
    _max_pages: usize,
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
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        let page_size = area.height as usize;
        let area_size = area.area() as usize;
        trace!(
            "Rendering dmesg: {:?}, page={} log_size={}",
            area,
            page_size,
            model.borrow_mut().dmesg.len()
        );
        // get last page_size entries from or the whole buffer if it's smaller
        let content = model
            .borrow_mut()
            .dmesg
            .iter()
            .rev()
            .take(page_size)
            .rev()
            .map(|entry| {
                if let Some(ts) = entry.timestamp_from_system_start {
                    format!("[{:.6}] {}\n", ts.as_secs_f32(), entry.message)
                } else {
                    // we've got a 'continuation' string in a format key=value
                    format!("\t{}\n", entry.message)
                }
            })
            .fold(String::with_capacity(area_size), |acc, e| acc + &e);

        Paragraph::new(content).render(*area, frame.buffer_mut())
    }
}

impl IWindow for DmesgViewer {}
impl IEventHandler for DmesgViewer {
    fn handle_event(&mut self, event: crate::events::Event) -> Option<crate::ui::action::Action> {
        match event {
            Event::Tick | Event::TerminalResize(_, _) => None, // we want this to trigger a rerender, but that will happen even if we do nothing here
            Event::Key(_) => None, // todo, but don't want crashing for the demo
        }
    }
}
