use crate::ui::focus_tracker::FocusMode;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::Tabs,
    Frame,
};
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    traits::{IPresenter, IWidget},
    ui::{focus_tracker::FocusTracker, window::LayoutMap},
};

use super::{tools::ElementHashMap, window::WidgetMap};

const NUM_FIELDS: usize = 5;

struct NetworkDialog {
    focus: FocusTracker,
    current_tab: NetworkTabs,
    layout: LayoutMap,
    old_rect: Rect,
    page_widgets: WidgetMap,
    ip_fields: Vec<Box<dyn IWidget>>,
    proxy_fields: Vec<Box<dyn IWidget>>,
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
enum NetworkTabs {
    #[default]
    IP,
    Proxy,
}

impl NetworkDialog {
    pub fn new() -> Self {
        let focus_order = vec!["a".to_string(), "b".to_string()];

        Self {
            focus: FocusTracker::create_from_taborder(focus_order, None, FocusMode::Wrap),
            layout: LayoutMap::new(),
            old_rect: Rect::ZERO,
            page_widgets: ElementHashMap::new(),
            ip_fields: Vec::new(),
            proxy_fields: Vec::new(),
            current_tab: NetworkTabs::IP,
        }
    }

    fn do_layout(&mut self, area: &Rect) {
        if self.old_rect == *area {
            return;
        }
        let [tabs, mode, fields, buttonbar] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(1),
        ])
        .areas(*area);

        let mut lm = LayoutMap::new();

        let _ = lm.add_or_update("tabs".to_string(), tabs.clone());
        let _ = lm.add_or_update("mode".to_string(), mode.clone());
        let _ = lm.add_or_update("fileds".to_string(), fields.clone());

        let [ok, cancel] = Layout::horizontal(vec![Constraint::Length(3); 2])
            .flex(Flex::Start)
            .areas(buttonbar);

        let _ = lm.add_or_update("ok".to_string(), ok);
        let _ = lm.add_or_update("cancel".to_string(), cancel);

        let field_rects: [Rect; NUM_FIELDS] =
            Layout::vertical(vec![Constraint::Length(3); NUM_FIELDS]).areas(fields);
        field_rects.iter().enumerate().for_each(|(i, f)| {
            lm.add_or_update(i.to_string(), *f);
            ()
        });

        self.layout = lm;
        // return self.layout.as_ref().unwrap();
    }

    fn render_main(&mut self, area: &Rect, frame: &mut Frame) {
        self.do_layout(area);
        // let layout = &self.as_ref().layout.unwrap();

        frame.render_widget(tabs(), self.layout["tabs"]);
        // frame.render_widget(tabs(), layout["tabs"]);

        let (mode_selector, field_list) = match self.current_tab {
            NetworkTabs::IP => ("ip_mode", &self.ip_fields),
            NetworkTabs::Proxy => ("proxy_mode", &self.proxy_fields),
        };

        self.page_widgets[mode_selector].render(&self.layout["mode"], frame, false);

        self.render_fields(
            &self.layout["fields"],
            frame,
            &field_list,
            self.focus.get_focused_view().unwrap(),
        )
    }

    fn render_fields(
        &self,
        area: &Rect,
        frame: &mut Frame,
        field_list: &Vec<Box<dyn IWidget>>,
        focused: String,
    ) {
        field_list
            .iter()
            .enumerate()
            .for_each(|(i, field)| field.render(area, frame, i.to_string() == focused))
    }
}

impl IPresenter for NetworkDialog {
    fn render(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        _model: &std::rc::Rc<crate::model::Model>,
        _focused: bool,
    ) {
        self.render_main(area, frame)
    }
}

fn tabs() -> Tabs<'static> {
    let tab_titles = NetworkTabs::iter().map(NetworkTabs::to_tab_title);
    // let block = Block::new();
    Tabs::new(tab_titles)
        // .block(block)
        .highlight_style(Modifier::REVERSED)
        .divider(" ")
        .padding("", "")
}

impl NetworkTabs {
    fn to_tab_title(self) -> Line<'static> {
        let text = self.to_string();
        format!(" {text} ").bg(Color::Black).into()
    }
}
