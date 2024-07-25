use crate::device::summary::DeviceSummary;
use ratatui::text::Line;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Wrap;
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
    state: DeviceSummary,
    layout: Option<LayoutMap>,
    old_size: Rect,
}

impl HomePage {
    pub fn new() -> Self {
        let hp = HomePage {
            layout: None,
            state: DeviceSummary::dummy_summary(),
            old_size: Rect::ZERO,
        };
        hp
    }
    pub fn do_layout(&self, area: &Rect, _model: &Rc<Model>) -> LayoutMap {
        let [left, right] =
            Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(*area);

        let [usb, pci] =
            Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(right);

        let mut lm = LayoutMap::new();
        let _ = lm.add_or_update("summary".to_string(), left.clone());
        let _ = lm.add_or_update("usb".to_string(), usb.clone());
        let _ = lm.add_or_update("pci".to_string(), pci.clone());
        lm
    }

    pub fn do_render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>) {
        if self.layout.is_none() || self.old_size != *area {
            self.layout = Some(self.do_layout(area, &model));
            self.old_size = area.clone();
        }
        let layout = self.layout.as_ref().unwrap();

        let left = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(format!("Name: {}", self.state.name)),
            Line::from(format!(
                "Last update: {}",
                self.state.last_checkin.format("%d-%m-%Y %H:%M %Z")
            )),
        ]))
        .block(Block::bordered().title("Device Summary"));
        frame.render_widget(left, layout["summary"]);

        let usb = Paragraph::new(Text::from(self.state.usb_devices.join("\n")))
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title("USB Devices"));
        frame.render_widget(usb, layout["usb"]);

        let pci = Paragraph::new(Text::from(self.state.pci_devices.join("\n")))
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title("PCI Devices"));
        frame.render_widget(pci, layout["pci"]);
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
