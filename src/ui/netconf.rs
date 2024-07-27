use std::collections::HashMap;

use crate::{
    events::Event::{self, Key},
    traits::{IElementEventHandler, IEventHandler, IWidgetPresenter, IWindow},
    ui::{action::UiActions, focus_tracker::FocusMode, widgets::button::ButtonElement},
};
use crossterm::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};
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
    traits::{IPresenter, IWidget},
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
    ip_fields: Vec<Box<dyn IWidget>>,
    proxy_fields: Vec<Box<dyn IWidget>>,
    interface_name: String,
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
enum NetworkTabs {
    #[default]
    IP,
    Proxy,
}

impl NetworkDialog {
    pub fn new() -> Self {
        let focus_order: Vec<String> = vec![
            "mode".to_string(),
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "ok".to_string(),
            "cancel".to_string(),
        ];

        let mut s = Self {
            focus: FocusTracker::create_from_taborder(
                focus_order,
                Some("mode".to_string()),
                FocusMode::Wrap,
            ),
            layout: HashMap::new(),
            old_rect: Rect::ZERO,
            page_widgets: HashMap::new(),
            ip_fields: Vec::new(),
            proxy_fields: Vec::new(),
            selected_tab: NetworkTabs::IP,
            interface_name: "Home".to_string(),
        };

        s.page_widgets.insert(
            "ip_mode".to_string(),
            Box::new(SpinBoxElement::new(vec!["static", "dynamic"])),
        );
        s.page_widgets.insert(
            "proxy_mode".to_string(),
            Box::new(SpinBoxElement::new(vec!["automatic", "manual"])),
        );

        s.ip_fields
            .push(Box::new(InputFieldElement::new("IP", None)));
        s.ip_fields
            .push(Box::new(InputFieldElement::new("Gateway", None)));
        s.ip_fields
            .push(Box::new(InputFieldElement::new("DNS", None)));
        s.ip_fields
            .push(Box::new(InputFieldElement::new("Domain", None)));

        s.proxy_fields
            .push(Box::new(InputFieldElement::new("HTTP", None)));
        s.proxy_fields
            .push(Box::new(InputFieldElement::new("HTTPS", None)));
        s.proxy_fields
            .push(Box::new(InputFieldElement::new("Socks", None)));
        s.proxy_fields
            .push(Box::new(InputFieldElement::new("Domain", None)));
        s
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

        let _ = lm.insert("tabs".to_string(), tabs.clone());
        let _ = lm.insert("mode".to_string(), mode.clone());
        let _ = lm.insert("fileds".to_string(), fields.clone());

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

        let (mode_selector, field_list) = match self.selected_tab {
            NetworkTabs::IP => ("ip_mode", &mut self.ip_fields),
            NetworkTabs::Proxy => ("proxy_mode", &mut self.proxy_fields),
        };

        self.page_widgets.get_mut(mode_selector).unwrap().render(
            &self.layout["mode"],
            frame,
            focused_element == "mode",
        );

        field_list.iter_mut().enumerate().for_each(|(i, field)| {
            field.render(
                &self.layout[&i.to_string()],
                frame,
                i.to_string() == self.focus.get_focused_view().unwrap(),
            )
        });

        // render the buttons
        ButtonElement::new("ok").render(&self.layout["ok"], frame, focused_element == "ok");
        ButtonElement::new("cancel").render(
            &self.layout["cancel"],
            frame,
            focused_element == "cancel",
        );
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
                    return Some(Action::new("edit network", UiActions::Redraw));
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right {
                    debug!("CTRL+Right: switching tab view");
                    self.selected_tab = self.selected_tab.next();
                    return Some(Action::new("edit network", UiActions::Redraw));
                }

                debug!("key pressed {:?}", key);
                let focus = self.focus.get_focused_view()?;
                debug!("focused view {}", focus);
                let widget = self.page_widgets.get_mut(&focus)?;
                debug!("widget found");
                let activity = widget.handle_key_event(key)?;
                match activity {
                    Activity::Action(action) => Some(Action::new("edit network", action)),
                    Activity::Event(_) => None, //todo input validation
                }
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
