use std::iter::Map;

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, Tabs, Widget}, Frame,
};
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    traits::{IPresenter, IWidget},
    ui::{focus_tracker::FocusTracker, window::LayoutMap},
};

use super::window::WidgetMap;


const num_fields = 5;

struct NetworkDialog {
    main_focus: FocusTracker,
    current_tab: NetworkTabs
    layout: Option<LayoutMap>,
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
    pub fn new() -> Self{
        Self{ focus: todo!(),
            layout: todo!(), old_rect: todo!(), page_widgets: todo!(), ip_fields: todo!(), proxy_fields: todo!() }
    }

    fn do_layout(&mut self, area: &Rect) -> &LayoutMap {
        if self.layout.as_ref().is_some() && self.old_rect == *area {
            return self.layout.as_ref().unwrap();
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

        let field_rects = Layout::vertical(vec![Constraint::Length(3); num_fields]).areas(fields);
        field_rects.iter().enumerate().for_each(|(i,f)|{lm.add_or_update(i.to_string() , *f);()});

        self.layout = Some(lm);
        return self.layout.as_ref().unwrap();
    }

    fn render_main(&mut self, area: &Rect, frame: &mut Frame){
        let layout = self.do_layout(area);

        frame.render_widget(tabs(), layout["tabs"]);

        match self.current_tab{
            NetworkTabs::IP => self.page_widgets.get("ip_mode").unwrap().render(&layout["mode"], frame, false),
            NetworkTabs::Proxy => ,
        }

    }

    fn render_fields(&self, area: &Rect, frame: &mut Frame, focused: u8){}
}

impl IPresenter for NetworkDialog {
    fn render(
        &mut self,
        area: &Rect,
        frame: &mut Frame<'_>,
        model: &std::rc::Rc<crate::model::Model>,
        focused: bool,
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
