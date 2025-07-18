// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{
    actions::MonActions,
    model::{
        device::network::{NetworkInterfaceStatus, ProxyConfig},
        model::Model,
    },
    traits::IWindow,
};

use super::{
    action::{Action, UiActions},
    tools::centered_rect,
    widgets::{
        button::ButtonElement, input_field::InputFieldElement, spin_box::SpinBoxElement,
        tab::TabElement,
    },
    window::Window,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ProxyType {
    None,
    Manual,
    // Pac,
    // Wad,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceState {
    pub iface_name: String,
    pub ip_dhcp: bool,
    pub proxy_type: ProxyType,
    pub ipv4: String,
    pub ipv6: String,
    pub mask: String,
    pub gw: String,
    pub proxy_url: String,
    pub proxy_certificate: String,
    pub pac_file: String,
    pub domain: String,
    pub dns: String,
    pub ntp: String,
    // manual proxies
    pub proxy_http: String,
    pub proxy_https: String,
    pub proxy_ftp: String,
    pub proxy_socks: String,
}

impl InterfaceState {
    pub fn is_dhcp(&self) -> bool {
        self.ip_dhcp
    }
}

// here we deal with Strings because we update them from InputFiled
#[derive(Clone, Debug, PartialEq)]
pub struct IpDialogState {
    selected_tab: String,
    focus_tarcker_state: HashMap<String, usize>,
    pub new_iface_state: InterfaceState,
    pub old_iface_state: InterfaceState,
}

impl IpDialogState {
    pub fn get_focused_view(&self) -> Option<usize> {
        self.focus_tarcker_state.get(&self.selected_tab).copied()
    }
    pub fn get_current_tab_order(&self) -> Vec<&str> {
        let mut order = match self.selected_tab.as_str() {
            "IP" => {
                if self.new_iface_state.ip_dhcp {
                    vec!["ip_spinner"]
                } else {
                    vec![
                        "ip_spinner",
                        "ipv4",
                        "ipv6",
                        "mask",
                        "gw",
                        "domain",
                        "dns",
                        "ntp",
                    ]
                }
            }
            "Proxy" => match self.new_iface_state.proxy_type {
                ProxyType::None => vec!["proxy_spinner"],
                ProxyType::Manual => {
                    vec![
                        "proxy_spinner",
                        "http",
                        "https",
                        "ftp",
                        "socks",
                        // this is not supported yet but let's keep it here for future use
                        // "certificate",
                        // "upload",
                    ]
                } // this is not supported yet but let's keep it here for future use
                  // ProxyType::Wad => vec!["proxy_spinner"],
                  // ProxyType::Pac => vec!["proxy_spinner", "pac_file", "upload"],
            },
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
    let current_tab_order = w
        .state
        .get_current_tab_order()
        .iter()
        .map(|s| s.to_string())
        .collect();
    w.set_focus_tracker_tab_order(current_tab_order);
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

    let index = if w.state.new_iface_state.ip_dhcp {
        0
    } else {
        1
    };
    w.add_widget(
        "ip_spinner",
        SpinBoxElement::new(vec!["DHCP", "Static"]).selected(index),
    );

    w.add_widget(
        "ipv4",
        InputFieldElement::new("IPv4", Some(w.state.new_iface_state.ipv4.as_str()))
            .with_text_hint("e.g. 192.168.0.1"),
    );

    w.add_widget(
        "ipv6",
        InputFieldElement::new("IPv6", Some(w.state.new_iface_state.ipv6.as_str()))
            .with_text_hint("e.g. c820::1"),
    );

    w.add_widget(
        "mask",
        InputFieldElement::new("Mask", Some(w.state.new_iface_state.mask.as_str()))
            .with_text_hint("w.g. 255.255.255.0"),
    );
    w.add_widget(
        "gw",
        InputFieldElement::new("Gateway", Some(w.state.new_iface_state.gw.as_str()))
            .with_text_hint("e.g. 192.168.1.1"),
    );
    w.add_widget(
        "dns",
        InputFieldElement::new("DNS", Some(w.state.new_iface_state.dns.as_str()))
            .with_text_hint("e.g. 1.1.1.1, 4.4.4.4"),
    );
    w.add_widget(
        "domain",
        InputFieldElement::new("Domain", Some(w.state.new_iface_state.domain.as_str()))
            .with_text_hint("e.g. example.com"),
    );
    w.add_widget(
        "ntp",
        InputFieldElement::new("NTP", Some(w.state.new_iface_state.ntp.as_str()))
            .with_text_hint("e.g. 94.130.23.46, pool.ntp.org"),
    );

    // proxy widgets
    w.add_widget(
        "proxy_spinner",
        SpinBoxElement::new(vec!["None", "Manual" /*, "Pac"*/]),
    );
    w.add_widget(
        "http",
        InputFieldElement::new("HTTP", Some(&w.state.new_iface_state.proxy_http.as_str())),
    );
    w.add_widget(
        "https",
        InputFieldElement::new("HTTPs", Some(&w.state.new_iface_state.proxy_https.as_str())),
    );
    w.add_widget(
        "ftp",
        InputFieldElement::new("FTP", Some(&w.state.new_iface_state.proxy_ftp.as_str())),
    );
    w.add_widget(
        "socks",
        InputFieldElement::new("SOCKS", Some(&w.state.new_iface_state.proxy_socks.as_str())),
    );
    // w.add_widget(
    //     "pac_file",
    //     InputFieldElement::new("PAC file", Some(&w.state.new_iface_state.pac_file.as_str()))
    //         .enabled(false),
    // );
    // This is not supported yet but let's keep it here for future use
    // w.add_widget(
    //     "certificate",
    //     InputFieldElement::new(
    //         "Proxy Certificcate",
    //         Some(&w.state.new_iface_state.proxy_certificate.as_str()),
    //     )
    //     .enabled(false),
    // );
    // w.add_widget("upload", ButtonElement::new("Upload"));
}

fn update_ip_layout(w: &mut Window<IpDialogState>, rect: &Rect) {
    debug!("update_ip_layout");
    // split dialog content area. Top - Spinner widget
    let [spinner_rect, input_rect] =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).areas(*rect);

    w.update_layout("ip_spinner", spinner_rect);

    if !w.state.new_iface_state.ip_dhcp {
        let [ip, ipv6, mask, gw, domain, dns, ntp] = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(input_rect);

        w.update_layout("ipv4", ip);
        w.update_layout("mask", mask);
        w.update_layout("gw", gw);
        w.update_layout("ipv6", ipv6);
        w.update_layout("domain", domain);
        w.update_layout("dns", dns);
        w.update_layout("ntp", ntp);
    }
}
fn update_proxy_layout(w: &mut Window<IpDialogState>, rect: &Rect) {
    debug!("update_proxy_layout");
    let [spinner_rect, input_rect] =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).areas(*rect);

    w.update_layout("proxy_spinner", spinner_rect);

    match w.state.new_iface_state.proxy_type {
        ProxyType::None => {}
        ProxyType::Manual => {
            let [http, https, ftp, socks, certificate] = Layout::vertical(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .areas(input_rect);

            let [cert_str, upload_button] =
                Layout::horizontal(vec![Constraint::Fill(1), Constraint::Length(10)])
                    .flex(Flex::End)
                    .areas(certificate);

            w.update_layout("http", http);
            w.update_layout("https", https);
            w.update_layout("ftp", ftp);
            w.update_layout("socks", socks);
            w.update_layout("certificate", cert_str);
            w.update_layout("upload", upload_button);
        } // This is not supported yet but let's keep it here for future use
          // ProxyType::Pac => {
          //     let [pac_file_area] = Layout::vertical(vec![Constraint::Length(3)]).areas(input_rect);
          //     let [pac_url, upload] =
          //         Layout::horizontal(vec![Constraint::Fill(1), Constraint::Length(10)])
          //             .flex(Flex::SpaceBetween)
          //             .areas(pac_file_area);
          //     w.update_layout("pac_file", pac_url);
          //     w.update_layout("upload", upload);
          // }
          // ProxyType::Wad => {}
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

fn ip_dialog_layout(w: &mut Window<IpDialogState>, rect: &Rect, _model: &Rc<Model>) {
    debug!("ip_dialog_layout. selected tab: {}", w.state.selected_tab);
    w.clear_layout();

    let rect = centered_rect(40, 80, *rect);
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
        .title(w.state.new_iface_state.iface_name.as_str());

    frame.render_widget(block, frame_rect);
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
                w.state.new_iface_state.ip_dhcp = *selected == 0;
                update_tab_order(w);
                Some(Action::new(source, UiActions::Redraw))
            }
            "proxy_spinner" => {
                w.state.new_iface_state.proxy_type = match *selected {
                    0 => ProxyType::None,
                    1 => ProxyType::Manual,
                    // 2 => ProxyType::Pac,
                    _ => ProxyType::None,
                };
                update_tab_order(w);
                Some(Action::new(source, UiActions::Redraw))
            }
            _ => None,
        },
        UiActions::ButtonClicked(name) => match name.as_str() {
            "cancel" => Some(Action::new(&w.name, UiActions::DismissDialog)),
            "ok" => Some(Action::new(
                &w.name,
                UiActions::AppAction(MonActions::NetworkInterfaceUpdated(
                    w.state.old_iface_state.clone(),
                    w.state.new_iface_state.clone(),
                )),
            )),
            _ => None,
        },
        UiActions::Input { text } => {
            match source.as_str() {
                "ipv4" => w.state.new_iface_state.ipv4 = text.clone(),
                "ipv6" => w.state.new_iface_state.ipv6 = text.clone(),
                "mask" => w.state.new_iface_state.mask = text.clone(),
                "gw" => w.state.new_iface_state.gw = text.clone(),
                "dns" => w.state.new_iface_state.dns = text.clone(),
                "domain" => w.state.new_iface_state.domain = text.clone(),
                "http" => w.state.new_iface_state.proxy_http = text.clone(),
                "https" => w.state.new_iface_state.proxy_https = text.clone(),
                "ftp" => w.state.new_iface_state.proxy_ftp = text.clone(),
                "socks" => w.state.new_iface_state.proxy_socks = text.clone(),
                "ntp" => w.state.new_iface_state.ntp = text.clone(),
                _ => {}
            }
            None
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
    update_tab_order(w);

    // and the focused view
    let focus_tracker_state = w.state.focus_tarcker_state.get(selected_tab);
    if let Some(focus_tracker_state) = focus_tracker_state {
        w.set_focused_view(*focus_tracker_state);
    }
}

fn update_tab_order(w: &mut Window<IpDialogState>) {
    let new_tab_order = w
        .state
        .get_current_tab_order()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    w.set_focus_tracker_tab_order(new_tab_order);
}

impl From<&NetworkInterfaceStatus> for IpDialogState {
    fn from(iface: &NetworkInterfaceStatus) -> Self {
        // take only the first ipv4  and ipv6 address
        // TODO: per Milan, we may get local IPs on interfaces
        // need to find out how to filter them out
        // but those are just to fill the dialog, so it's not a big deal
        // user will change them anyway
        let ipv4 = iface
            .ipv4
            .as_ref()
            .map(|ipv4: &Vec<std::net::Ipv4Addr>| ipv4.first().cloned())
            .flatten()
            .map(|addr| addr.to_string())
            .unwrap_or_default();

        let ipv6 = iface
            .ipv6
            .as_ref()
            .map(|ipv6: &Vec<std::net::Ipv6Addr>| ipv6.first().cloned())
            .flatten()
            .map(|addr| addr.to_string())
            .unwrap_or_default();

        let proxy_type = match iface.proxy_config {
            ProxyConfig::None => ProxyType::None,
            ProxyConfig::Manual { .. } => ProxyType::Manual,
            // ProxyConfig::Pac { .. } => ProxyType::Pac,
            // ProxyConfig::Wad { .. } => ProxyType::Wad,
            _ => ProxyType::None,
        };

        let proxy_url = if let ProxyConfig::Wad { url, .. } = &iface.proxy_config {
            url.to_string()
        } else {
            "".to_string()
        };

        let pac_file = if let ProxyConfig::Pac { url, .. } = &iface.proxy_config {
            url.to_string()
        } else {
            "".to_string()
        };

        let mut proxy_ftp = "".to_string();
        let mut proxy_http = "".to_string();
        let mut proxy_https = "".to_string();
        let mut proxy_socks = "".to_string();

        if let ProxyConfig::Manual {
            http,
            https,
            ftp,
            socks,
        } = &iface.proxy_config
        {
            proxy_ftp = ftp.as_ref().map(|p| p.to_url()).unwrap_or_default();
            proxy_http = http.as_ref().map(|p| p.to_url()).unwrap_or_default();
            proxy_https = https.as_ref().map(|p| p.to_url()).unwrap_or_default();
            proxy_socks = socks.as_ref().map(|p| p.to_url()).unwrap_or_default();
        }

        // convert to comma separated string
        let dns = iface
            .dns
            .iter()
            .flatten()
            .map(|ip| ip.to_string())
            .collect::<Vec<String>>()
            .join(",");

        // same for NTP
        let ntp = iface
            .ntp_servers
            .iter()
            .flatten()
            .map(|ip| ip.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let domain = iface.domain.clone().unwrap_or_default();

        let new_iface_state = InterfaceState {
            iface_name: iface.name.clone(),
            ip_dhcp: iface.is_dhcp,
            ipv4: ipv4.clone(),
            ipv6: ipv6.clone(),
            proxy_type,
            mask: iface
                .subnet
                .map(|ip| ip.netmask().to_string())
                .unwrap_or_default(),
            gw: iface.gw.map(|ip| ip.to_string()).unwrap_or_default(),
            proxy_url,
            proxy_certificate: "".to_string(),
            pac_file,
            domain,
            dns,
            ntp,
            proxy_ftp,
            proxy_http,
            proxy_https,
            proxy_socks,
        };

        let old_iface_state = new_iface_state.clone();

        IpDialogState {
            selected_tab: "IP".to_string(),
            focus_tarcker_state: HashMap::new(),
            new_iface_state,
            old_iface_state,
        }
    }
}

pub fn create_ip_dialog(iface: &NetworkInterfaceStatus) -> impl IWindow {
    let state = IpDialogState::from(iface);
    let w = Window::builder("IP configuration")
        .with_layout(ip_dialog_layout)
        .with_render(ip_dialog_render)
        .with_on_child_ui_action(on_child_ui_action)
        .with_on_key_event(on_key_event)
        .with_on_init(on_init)
        .with_state(state)
        .build()
        .unwrap();
    w
}
