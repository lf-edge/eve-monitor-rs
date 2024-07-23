use crate::model::Model;
use std::collections::VecDeque;
use std::rc::Rc;

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
    lines_per_page: u16,
}

#[derive(Default, Debug)]
enum DmsgMode {
    #[default]
    Follow,
    Page,
}

impl DmesgViewer {
    pub fn new() -> Self {
        let mut def = DmesgViewer::default();
        def.lines_per_page = 120; // specially chosen to fill a normal terminal size for the demo. Should be chosen based on terminal height
        def
    }
}

impl IPresenter for DmesgViewer {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _model: &Rc<Model>, _focused: bool) {
        match rmesg::log_entries(rmesg::Backend::Default, false) {
            Err(err) => {
                error!("{}", err.to_string());
                Paragraph::new(err.to_string()).render(*area, frame.buffer_mut())
            }
            Ok(mut entries) => {
                self.lines_per_page = area.height;

                let page_list = entries.split_off(
                    entries
                        .len()
                        .saturating_sub((self.lines_per_page * 3).into()),
                );
                let page_contents = page_list
                    .into_iter()
                    .map(|entry| {
                        if let Some(ts) = entry.timestamp_from_system_start {
                            format!("[{:.6}] {}\n", ts.as_secs_f32(), entry.message)
                        } else {
                            "".to_string()
                        }
                    })
                    .reduce(|page, e| page + &e)
                    .unwrap();
                Paragraph::new(page_contents).render(*area, frame.buffer_mut())
            }
        };
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
