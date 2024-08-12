use std::{collections::HashMap, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{model::Model, traits::IWindow};

use super::{
    action::{Action, UiActions},
    tools::centered_rect,
    widgets::{
        button::ButtonElement, input_field::InputFieldElement, spin_box::SpinBoxElement,
        tab::TabElement,
    },
    window::Window,
};

#[derive(Default)]
pub struct IpDialogState {
    iface_name: String,
    selected_tab: String,
    focus_tarcker_state: HashMap<String, usize>,
    ip_dhcp: bool,
}

impl IpDialogState {
    pub fn get_focused_view(&self) -> Option<usize> {
        self.focus_tarcker_state.get(&self.selected_tab).copied()
    }
    pub fn get_current_tab_order(&self) -> Vec<&str> {
        let mut order = match self.selected_tab.as_str() {
            "IP" => {
                if self.ip_dhcp {
                    vec!["ip_spinner"]
                } else {
                    vec!["ip_spinner", "ip", "mask", "gw"]
                }
            }
            "Proxy" => vec!["proxy_spinner", "url"],
            _ => vec![],
        };
        order.push("ok");
        order.push("cancel");
        order
    }
}

fn on_init(w: &mut Window<IpDialogState>) {
    create_widgets(w);
    init_focus_tracker(w);
}

fn init_focus_tracker(w: &mut Window<IpDialogState>) {
    w.state.focus_tarcker_state.insert("IP".to_string(), 0);
    w.state.focus_tarcker_state.insert("Proxy".to_string(), 0);
    let currect_tab_order = w
        .state
        .get_current_tab_order()
        .iter()
        .map(|s| s.to_string())
        .collect();
    w.set_focus_tracker_tab_order(currect_tab_order);
    if let Some(focused_view) = w.state.get_focused_view() {
        w.set_focused_view(focused_view);
    }
}

fn create_widgets(w: &mut Window<IpDialogState>) {
    // create all widgets only once. We draw only widgets that present in the layout
    w.add_widget(
        "tabs",
        TabElement::new(
            vec!["IP", "Proxy"],
            "IP",
            Some(" Use ctrl + ◄ ► to change tab"),
        ),
    );

    // buttons
    w.add_widget("ok", ButtonElement::new("ok"));
    w.add_widget("cancel", ButtonElement::new("cancel"));

    let index = if w.state.ip_dhcp { 0 } else { 1 };
    w.add_widget(
        "ip_spinner",
        SpinBoxElement::new(vec!["DHCP", "Static"]).selected(index),
    );
    w.add_widget("ip", InputFieldElement::new("IPv4", Some("192.168.1.1")));
    w.add_widget(
        "mask",
        InputFieldElement::new("Mask", Some("255.255.255.0")),
    );
    w.add_widget("gw", InputFieldElement::new("Gateway", Some("192.168.0.1")));

    // proxy widgets
    w.add_widget("proxy_spinner", SpinBoxElement::new(vec!["None", "Manual"]));
    w.add_widget(
        "url",
        InputFieldElement::new("URL", Some("http://proxy.com")),
    );
}

fn update_ip_layout(w: &mut Window<IpDialogState>, rect: &Rect) {
    debug!("update_ip_layout");
    // split dialog content area. Top - Spinner widget
    let [spinner_rect, input_rect] =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).areas(*rect);

    w.update_layout("ip_spinner", spinner_rect);

    if !w.state.ip_dhcp {
        let [ip, mask, gw] = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(input_rect);

        w.update_layout("ip", ip);
        w.update_layout("mask", mask);
        w.update_layout("gw", gw);
        //w.set_focus_tracker_tab_order(vec!["ip_spinner", "ip", "mask", "gw"]);
        //w.set_focused_view(w.state.focus_tarcker_state["IP"]);
    }
}
fn update_proxy_layout(w: &mut Window<IpDialogState>, rect: &Rect) {
    debug!("update_proxy_layout");
    let [spinner_rect, input_rect] =
        Layout::vertical(vec![Constraint::Length(3), Constraint::Fill(1)]).areas(*rect);

    w.update_layout("proxy_spinner", spinner_rect);

    if !w.state.ip_dhcp {
        let [url, ip, mask, gw] = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(input_rect);

        w.update_layout("url", url);
        // w.update_layout("mask", mask);
        // w.update_layout("gw", gw);
        // w.set_focus_tracker_tab_order(vec!["ip_spinner", "ip", "mask", "gw"]);
        //w.set_focused_view(w.state.focus_tarcker_state["Proxy"]);
    }
}

fn update_current_layout(w: &mut Window<IpDialogState>, rect: &Rect) {
    match w.state.selected_tab.as_str() {
        "IP" => {
            update_ip_layout(w, rect);
        }
        "Proxy" => {
            update_proxy_layout(w, rect);
        }
        _ => {}
    }
}

fn ip_dialog_layout(w: &mut Window<IpDialogState>, rect: &Rect, model: &Rc<Model>) {
    debug!("ip_dialog_layout. selected tab: {}", w.state.selected_tab);
    w.clear_layout();

    let rect = centered_rect(60, 60, *rect);
    let content_with_buttons = rect.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    w.update_layout("frame", rect);

    // split content are
    let [dialog_content, buttons] =
        Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::End)
            .areas(content_with_buttons);

    // split dialog content area. Top - Tab widget
    let [tabs, dialog_content_rect] =
        Layout::vertical(vec![Constraint::Length(3), Constraint::Fill(1)]).areas(dialog_content);
    w.update_layout("tabs", tabs);

    update_current_layout(w, &dialog_content_rect);

    // buttons
    let [ok, cancel] = Layout::horizontal(vec![Constraint::Length(6), Constraint::Length(10)])
        .flex(Flex::End)
        .areas(buttons);
    w.update_layout("ok", ok);
    w.update_layout("cancel", cancel);
}

fn ip_dialog_render(
    w: &mut Window<IpDialogState>,
    _rect: &Rect,
    frame: &mut Frame<'_>,
    _model: &Rc<Model>,
) {
    // render frame
    let frame_rect = w.get_layout("frame");

    // clear area under the dialog
    let clear = Clear {};
    frame.render_widget(clear, frame_rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black))
        .title(w.state.iface_name.as_str());

    frame.render_widget(block, frame_rect);

    // // render debug rect
    //     w.update_layout("dialog_content_rect", dialog_content_rect);
    // let debug_rect = w.get_layout("dialog_content_rect");
    // let block = Block::default()
    //     .borders(Borders::ALL)
    //     .border_type(BorderType::Rounded)
    //     .border_style(Style::default().fg(Color::White))
    //     .style(Style::default().bg(Color::Black))
    //     .title("Debug");
    // frame.render_widget(block, debug_rect);
}

fn on_key_event(w: &mut Window<IpDialogState>, key: KeyEvent) -> Option<Action> {
    debug!("ip_dialog: on_key_event");

    if key.code == KeyCode::Esc {
        return Some(Action::new(&w.name, UiActions::DismissDialog));
    }

    Some(Action::new(
        "tabs",
        w.get_widget_mut("tabs").unwrap().handle_key_event(key)?,
    ))
}

fn on_child_ui_action(
    w: &mut Window<IpDialogState>,
    source: &String,
    action: &UiActions,
) -> Option<Action> {
    debug!("on_child_ui_action: {}:{:?}", source, action);
    match action {
        UiActions::TabChanged(old_tab, selected_tab) => {
            save_restore_ft_state(w, old_tab, selected_tab);
            Some(Action::new(source, UiActions::Redraw))
        }
        UiActions::SpinBox { selected } => match source.as_str() {
            "ip_spinner" => {
                w.state.ip_dhcp = *selected == 0;
                Some(Action::new(source, UiActions::Redraw))
            }
            _ => None,
        },
        UiActions::ButtonClicked(name) if name == "cancel" => {
            Some(Action::new(&w.name, UiActions::DismissDialog))
        }
        _ => None,
    }
}

fn save_restore_ft_state(w: &mut Window<IpDialogState>, old_tab: &String, selected_tab: &String) {
    // save FocusTracker state for the old tab
    w.state
        .focus_tarcker_state
        .insert(old_tab.clone(), w.get_focused_view());

    w.state.selected_tab = selected_tab.clone();

    // restore FocusTracker state for the new tab
    // update tab order
    let new_tab_order = w
        .state
        .get_current_tab_order()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    w.set_focus_tracker_tab_order(new_tab_order);

    // and the focused view
    let focus_tracker_state = w.state.focus_tarcker_state.get(selected_tab);
    if let Some(focus_tracker_state) = focus_tracker_state {
        w.set_focused_view(*focus_tracker_state);
    }
}

pub fn create_ip_dialog() -> impl IWindow {
    let w = Window::builder("IP configuration")
        .with_layout(ip_dialog_layout)
        .with_render(ip_dialog_render)
        .with_on_child_ui_action(on_child_ui_action)
        .with_on_key_event(on_key_event)
        .with_on_init(on_init)
        .with_state(IpDialogState {
            iface_name: "example".to_string(),
            selected_tab: "IP".to_string(),
            ..Default::default()
        })
        .build()
        .unwrap();
    w
}
