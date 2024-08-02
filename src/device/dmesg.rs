use crate::model::Model;
use crate::ui::action::Action;
use crate::ui::activity::Activity;
use std::cmp;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::events::Event;
use crate::traits::{IEventHandler, IPresenter, IWindow};
use crossterm::event::{KeyCode, KeyEvent};
use log::trace;
use log2::error;
use ratatui::prelude::Rect;
use ratatui::widgets::{Clear, Paragraph, Widget};
use ratatui::Frame;
use rmesg::entry::Entry;

const MAX_LINES: usize = 1000;

#[derive(Debug, Default)]
pub struct DmesgViewer {
    _mode: DmsgMode,
    buffer_index: usize,
    lines_per_page: u16,
    buffer_len: usize,
}

#[derive(Default, Debug)]
enum DmsgMode {
    #[default]
    Follow,
    Scroll,
}

impl DmesgViewer {
    pub fn new() -> Self {
        DmesgViewer::default()
    }

    fn switch_to_scroll_mode(&mut self) {
        self._mode = DmsgMode::Scroll;
    }

    pub fn handle_keys_following(&mut self, key: KeyEvent) -> Option<Activity> {
        match key.code {
            KeyCode::Down
            | KeyCode::Up
            | KeyCode::PageDown
            | KeyCode::PageUp
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Char(' ') => {
                self.switch_to_scroll_mode();
                self.handle_keys_scroll(key)
            }
            _ => None,
        }
    }

    pub fn handle_keys_scroll(&mut self, key: KeyEvent) -> Option<Activity> {
        match key.code {
            KeyCode::Down => {
                self.buffer_index = cmp::min(
                    self.buffer_index + 1 as usize,
                    self.buffer_len - self.lines_per_page as usize,
                );
            }
            KeyCode::Up => {
                self.buffer_index = self.buffer_index.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.buffer_index = cmp::min(
                    self.buffer_index + self.lines_per_page as usize,
                    self.buffer_len - self.lines_per_page as usize,
                );
            }
            KeyCode::PageUp => {
                self.buffer_index = self
                    .buffer_index
                    .saturating_sub(self.lines_per_page as usize);
            }
            KeyCode::End => {
                self.buffer_index = self.buffer_len - self.lines_per_page as usize;
            }
            KeyCode::Home => {
                self.buffer_index = 0;
            }
            KeyCode::Char(' ') => {
                self._mode = DmsgMode::Follow;
            }
            _ => return None,
        }
        Some(Activity::redraw())
    }
}

impl IPresenter for DmesgViewer {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        let page_size = area.height as usize;
        let area_size = area.area() as usize;
        self.buffer_len = model.borrow().dmesg.len();
        self.lines_per_page = area.height;
        trace!(
            "Rendering dmesg: {:?}, page={} log_size={}",
            area,
            page_size,
            model.borrow().dmesg.len()
        );

        let dmesg = &model.borrow().dmesg;
        // get last page_size entries from or the whole buffer if it's smaller
        let content: Vec<&Entry> = match self._mode {
            DmsgMode::Follow => {
                self.buffer_index = self.buffer_len.saturating_sub(page_size);
                dmesg.iter().rev().take(page_size).rev().collect()
            }
            DmsgMode::Scroll => dmesg
                .iter()
                .skip(self.buffer_index)
                .take(page_size)
                .collect(),
        };

        Paragraph::new(
            content
                .iter()
                .map(|entry| {
                    if let Some(ts) = entry.timestamp_from_system_start {
                        format!("[{:.6}] {}\n", ts.as_secs_f32(), entry.message)
                    } else {
                        // we've got a 'continuation' string in a format key=value
                        format!("  {}\n", entry.message)
                    }
                })
                .fold(String::with_capacity(area_size), |acc, e| acc + &e),
        )
        .render(*area, frame.buffer_mut());
    }
    //
    // fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _model: &Rc<Model>, _focused: bool) {
    //     self.lines_per_page = area.height;

    //     match rmesg::log_entries(rmesg::Backend::Default, false) {
    //         Err(err) => {
    //             error!("{}", err.to_string());
    //             Paragraph::new(err.to_string()).render(*area, frame.buffer_mut())
    //         }
    //         Ok(mut entries) => {
    //             let page_list = entries.split_off(
    //                 entries
    //                     .len()
    //                     .saturating_sub((self.lines_per_page * 3).into()),
    //             );
    //             let page_contents = page_list
    //                 .into_iter()
    //                 .map(|entry| {
    //                     if let Some(ts) = entry.timestamp_from_system_start {
    //                         format!("[{:.6}] {}\n", ts.as_secs_f32(), entry.message)
    //                     } else {
    //                         "".to_string()
    //                     }
    //                 })
    //                 .reduce(|page, e| page + &e)
    //                 .unwrap();
    //             Paragraph::new(page_contents).render(*area, frame.buffer_mut())
    //         }
    //     };
    // }
}

impl IWindow for DmesgViewer {}
impl IEventHandler for DmesgViewer {
    fn handle_event(&mut self, event: crate::events::Event) -> Option<Action> {
        let activity = match event {
            Event::Tick | Event::TerminalResize(_, _) => None, // we want this to trigger a rerender, but that will happen even if we do nothing here
            Event::Key(key) => match self._mode {
                DmsgMode::Follow => self.handle_keys_following(key),
                DmsgMode::Scroll => self.handle_keys_scroll(key),
            }, // todo, but don't want crashing for the demo
        }?;
        match activity {
            Activity::Action(action) => Some(Action::new("something".to_string(), action)),
            Activity::Event(_) => None,
        }
    }
}
