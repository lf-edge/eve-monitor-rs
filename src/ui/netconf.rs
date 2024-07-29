use std::{cell::LazyCell, collections::HashMap};

use crate::{
    events::Event::{self, Key},
    traits::{IElementEventHandler, IEventHandler, IWidgetPresenter, IWindow},
    ui::{action::UiActions, focus_tracker::FocusMode, widgets::button::ButtonElement},
};
use crossterm::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use futures::stream::Collect;
use log::debug;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Tabs, Widget},
    Frame,
};
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    traits::IPresenter,
    ui::{focus_tracker::FocusTracker, window::LayoutMap},
};

use super::{
    action::Action,
    activity::Activity,
    widgets::{input_field::InputFieldElement, spin_box::SpinBoxElement},
    window::WidgetMap,
};

const NUM_FIELDS: usize = 5;

pub struct NetworkDialog {
    focus: FocusTracker,
    selected_tab: NetworkTabs,
    layout: LayoutMap,
    old_rect: Rect,
    page_widgets: WidgetMap,
    tab_widgets: HashMap<NetworkTabs, WidgetMap>,
    // ip_fields: WidgetMap,
    // proxy_fields: WidgetMap,
    interface_name: String,
}

#[derive(
    Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount, Hash, Eq, PartialEq,
)]
enum NetworkTabs {
    #[default]
    IP,
    Proxy,
}

const window_focus_order: LazyCell<Vec<String>> =
    LazyCell::new(|| vec!["ok".to_string(), "cancel".to_string()]);

impl NetworkDialog {
    pub fn new() -> Self {
        let mut page_widgets = WidgetMap::new();
        page_widgets.insert("ok".to_string(), Box::new(ButtonElement::new("ok")));
        page_widgets.insert("cancel".to_string(), Box::new(ButtonElement::new("cancel")));

        let mut ip_fields = WidgetMap::new();
        ip_fields.insert(
            "mode".to_string(),
            Box::new(SpinBoxElement::new(vec!["static", "dynamic"])),
        );
        ip_fields.insert(
            "ip".to_string(),
            Box::new(InputFieldElement::new("IP", Some(&"".to_string()))),
        );
        ip_fields.insert(
            "gateway".to_string(),
            Box::new(InputFieldElement::new("Gateway", Some(&"".to_string()))),
        );
        ip_fields.insert(
            "dns".to_string(),
            Box::new(InputFieldElement::new("DNS", Some(&"".to_string()))),
        );
        ip_fields.insert(
            "ip-domain".to_string(),
            Box::new(InputFieldElement::new("Domain", Some(&"".to_string()))),
        );

        let mut proxy_fields = WidgetMap::new();
        proxy_fields.insert(
            "mode".to_string(),
            Box::new(SpinBoxElement::new(vec!["automatic", "manual"])),
        );
        proxy_fields.insert(
            "proxy-http".to_string(),
            Box::new(InputFieldElement::new("HTTP", Some(&"".to_string()))),
        );
        proxy_fields.insert(
            "proxy-https".to_string(),
            Box::new(InputFieldElement::new("HTTPS", Some(&"".to_string()))),
        );
        proxy_fields.insert(
            "socks".to_string(),
            Box::new(InputFieldElement::new("Socks", Some(&"".to_string()))),
        );
        proxy_fields.insert(
            "proxy-domain".to_string(),
            Box::new(InputFieldElement::new("Domain", Some(&"".to_string()))),
        );

        let mut focus_order: Vec<String> = window_focus_order.clone();
        let mut ip_focus_order: Vec<String> =
            ip_fields.keys().into_iter().map(|s| (*s).clone()).collect();
        focus_order.append(&mut ip_focus_order);

        let focus = FocusTracker::create_from_taborder(
            focus_order,
            Some("mode".to_string()),
            FocusMode::Wrap,
        );

        let mut tab_widgets = HashMap::new();
        tab_widgets.insert(NetworkTabs::IP, ip_fields);
        tab_widgets.insert(NetworkTabs::Proxy, proxy_fields);

        Self {
            focus,
            layout: HashMap::new(),
            old_rect: Rect::ZERO,
            page_widgets,
            tab_widgets,
            selected_tab: NetworkTabs::IP,
            interface_name: "Home".to_string(),
        }
    }

    fn update_focus_order(&mut self) {
        let mut tab_order = window_focus_order.clone();
        self.tab_widgets[&self.selected_tab]
            .keys()
            .into_iter()
            .for_each(|key| tab_order.push(key.clone()));
        self.focus.set_tab_order(tab_order);
    }

    fn do_layout(&mut self, area: &Rect) {
        if self.old_rect == *area {
            return;
        }
        let [tabs, mode, fields, buttonbar] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(3),
        ])
        .margin(1)
        .areas(*area);

        let mut lm = LayoutMap::new();

        let _ = lm.insert("tabs".to_string(), tabs);
        let _ = lm.insert("mode".to_string(), mode);
        let _ = lm.insert("fileds".to_string(), fields);

        let [ok, cancel] = Layout::horizontal(vec![Constraint::Length(3); 2])
            .flex(Flex::Start)
            .areas(buttonbar);

        let _ = lm.insert("ok".to_string(), ok);
        let _ = lm.insert("cancel".to_string(), cancel);

        let field_rects: [Rect; NUM_FIELDS] =
            Layout::vertical(vec![Constraint::Length(3); NUM_FIELDS]).areas(fields);
        field_rects.iter().enumerate().for_each(|(i, f)| {
            lm.insert(i.to_string(), *f);
            ()
        });

        lm.insert("ip".to_string(), field_rects[0]);
        lm.insert("gateway".to_string(), field_rects[1]);
        lm.insert("dns".to_string(), field_rects[2]);
        lm.insert("ip-domain".to_string(), field_rects[3]);
        lm.insert("proxy-http".to_string(), field_rects[0]);
        lm.insert("proxy-https".to_string(), field_rects[1]);
        lm.insert("socks".to_string(), field_rects[2]);
        lm.insert("proxy-domain".to_string(), field_rects[3]);

        self.layout = lm;
        // return self.layout.as_ref().unwrap();
    }

    fn render_main(&mut self, area: &Rect, frame: &mut Frame) {
        self.do_layout(area);
        Clear.render(*area, frame.buffer_mut());
        let focused_element = self
            .focus
            .get_focused_view()
            .or(Some("".to_string()))
            .unwrap();

        debug!("focused element: {focused_element}");

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.interface_name.as_str());

        block.render(*area, frame.buffer_mut());

        frame.render_widget(
            tabs().select(self.selected_tab as usize),
            self.layout["tabs"],
        );

        self.page_widgets.iter_mut().for_each(|(name, field)| {
            field.render(
                &self.layout[name],
                frame,
                name.eq(&self.focus.get_focused_view().unwrap()),
            )
        });

        self.tab_widgets
            .get_mut(&self.selected_tab)
            .unwrap()
            .iter_mut()
            .for_each(|(name, field)| {
                field.render(
                    &self.layout[name],
                    frame,
                    name.eq(&self.focus.get_focused_view().unwrap()),
                )
            });
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
impl IWindow for NetworkDialog {}
impl IEventHandler for NetworkDialog {
    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Key(key) => {
                debug!("netconf edit dialog handling {:?}", key);
                if let Some(redraw) = self.focus.handle_key_event(key) {
                    return Some(Action::new("edit network", redraw));
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left {
                    debug!("CTRL+Left: switching tab view");
                    self.selected_tab = self.selected_tab.previous();
                    self.update_focus_order();
                    return Some(Action::new("edit network", UiActions::Redraw));
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right {
                    debug!("CTRL+Right: switching tab view");
                    self.selected_tab = self.selected_tab.next();
                    self.update_focus_order();
                    // self.focus.set_focus("mode");
                    return Some(Action::new("edit network", UiActions::Redraw));
                }

                debug!("key pressed {:?}", key);
                let focus = self.focus.get_focused_view()?;
                debug!("focused view {}", focus);
                if let Some(widget) = self.page_widgets.get_mut(&focus) {
                    debug!("widget found");
                    if let Some(activity) = widget.handle_key_event(key) {
                        return match activity {
                            Activity::Action(action) => Some(Action::new("edit network", action)),
                            Activity::Event(_) => None, //todo input validation
                        };
                    }
                }

                let tab_widgets = &mut self.tab_widgets.get_mut(&self.selected_tab)?;

                let widget = tab_widgets.get_mut(&focus)?;
                debug!("widget found {}", focus);
                if let Some(activity) = widget.handle_key_event(key) {
                    return match activity {
                        Activity::Action(action) => Some(Action::new("edit network", action)),
                        Activity::Event(_) => None, //todo input validation
                    };
                }

                None
            }
            Event::Tick | Event::TerminalResize(_, _) => None,
        }
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

    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}
