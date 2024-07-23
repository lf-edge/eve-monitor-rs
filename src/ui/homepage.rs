use std::rc::Rc;

use crate::events;
use crate::model::Model;
use crate::traits::{IEventHandler, IPresenter, IWindow};
use crate::ui::action::Action;
use crate::ui::window::LayoutMap;
use log::debug;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::prelude::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct HomePage {
    state: SummaryState,
    layout: Option<LayoutMap>,
}

#[derive(Clone, Debug)]
struct SummaryState {
    a: i32,
    ip: String,
}

impl HomePage {
    pub fn new() -> Self {
        let hp = HomePage {
            layout: None,
            state: SummaryState {
                a: 1,
                ip: "thing".to_string(),
            },
        };
        hp
    }
    pub fn do_layout(&self, area: &Rect, model: &Rc<Model>) -> LayoutMap {
        let chunks =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(*area);
        let [left, right] = [chunks[0], chunks[1]];

        let mut lm = LayoutMap::new();
        let _ = lm.add_or_update("left".to_string(), left.clone());
        let _ = lm.add_or_update("right".to_string(), right.clone());
        lm
    }

    pub fn do_render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>) {
        if self.layout.is_none() {
            self.layout = Some(self.do_layout(area, &model));
        }
        let layout = self.layout.as_ref().unwrap();

        let left = Paragraph::new(format!("{0:?}", self.state));
        frame.render_widget(left, layout["left"]);
    }
}

impl IPresenter for HomePage {
    // add code here
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _: bool) {
        self.do_render(area, frame, model);
    }
    fn can_focus(&self) -> bool {
        false
    }
}

impl IEventHandler for HomePage {
    fn handle_event(&mut self, event: events::Event) -> Option<Action> {
        debug!("HomePage handle_event {:?}", event);
        None
    }
}

impl IWindow for HomePage {}
