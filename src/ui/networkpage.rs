use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Padding, Paragraph, Row,
        StatefulWidget, Table, TableState,
    },
    Frame,
};

use crate::{
    events::Event,
    model::device::network::{NetworkInterfaceStatus, NetworkType},
    model::model::{Model, MonitorModel},
    traits::{IEventHandler, IPresenter, IWindow},
};

use super::{
    action::{Action, UiActions},
    traits::ISelector,
};

const MAC_LENGTH: u16 = 17;
const LINK_STATE_LENGTH: u16 = 4;
const IPV6_AVERAGE_LENGTH: u16 = 25;
const IFACE_LABEL_LENGTH: u16 = 10;

#[derive(Default)]
struct NetworkPage {
    list: InterfaceList,
    interface_names: Vec<String>,
}

struct InterfaceList {
    state: TableState,
    size: usize,
}

impl Default for InterfaceList {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            size: 0,
        }
    }
}

impl IWindow for NetworkPage {}

fn info_row_from_iface<'a, 'b>(iface: &'a NetworkInterfaceStatus) -> Row<'b> {
    // cells #1,2 IFace name and Link status
    let mut cells = vec![
        Cell::from(iface.name.clone()),
        if iface.up {
            Cell::from("UP").style(Style::new().green())
        } else {
            Cell::from("DOWN").style(Style::new().red())
        },
    ];

    // collect IP addresses and add as multiline
    let ipv4_len = iface.ipv4.as_ref().map_or(0, |v| v.len());
    let ipv6_len = iface.ipv6.as_ref().map_or(0, |v| v.len());

    let height = (ipv4_len + ipv6_len).max(1);

    // join Ipv4 and Ipv6 addresses and separate by newline
    let combined_ip_list_iter = iface
        .ipv4
        .iter()
        .flat_map(|v| v.iter())
        .map(|ip| ip.to_string())
        .chain(
            iface
                .ipv6
                .iter()
                .flat_map(|v| v.iter())
                .map(|ip| ip.to_string()),
        )
        .collect::<Vec<_>>()
        .join("\n");

    // cell #3 IP address list
    if height > 0 {
        cells.push(Cell::from(combined_ip_list_iter).style(Style::new().white()));
    } else {
        cells.push(Cell::from("N/A").style(Style::new().red()));
    }

    // cell #4 MAC
    cells.push(Cell::from(iface.mac.to_string()).style(Style::new().yellow()));

    Row::new(cells).height(height as u16)
}

fn details_table_from_iface<'a, 'b>(iface: &'a NetworkInterfaceStatus) -> Vec<Row<'b>> {
    // Row 0: Interface type
    // //FIXME: doesn't work reliably
    let iface_type = iface.media.to_string();
    let iface_type_row = Row::new(vec![
        Cell::from("Type").style(Style::new().yellow()),
        Cell::from(iface_type).style(Style::new().white()),
    ]);

    // IP type: DHCP/static
    let ip_source = if iface.is_dhcp { "DHCP" } else { "Static" };
    let ip_source_row = Row::new(vec![
        Cell::from("IP source").style(Style::new().yellow()),
        Cell::from(ip_source).style(Style::new().white()),
    ]);

    // Row 1: DNS
    let dns = iface.dns.as_ref().map_or_else(
        || "N/A".to_string(),
        |list| {
            list.iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    let dns_row_height = iface.dns.as_ref().map_or(1, |v| v.len());
    let dns_row = Row::new(vec![
        Cell::from("DNS").style(Style::new().yellow()),
        Cell::from(dns).style(Style::new().white()),
    ])
    .height(dns_row_height as u16);
    // Row 2: Gateway
    let gateway = iface
        .gw
        .as_ref()
        .map_or("N/A".to_string(), |v| v.to_string());
    let gateway_row = Row::new(vec![
        Cell::from("Gateway").style(Style::new().yellow()),
        Cell::from(gateway).style(Style::new().white()),
    ]);

    // Row 3: NTP
    let ntp = iface.ntp_servers.as_ref().map_or_else(
        || "N/A".to_string(),
        |list| {
            list.iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    let ntp_row_height = iface.ntp_servers.as_ref().map_or(1, |v| v.len());
    let ntp_row = Row::new(vec![
        Cell::from("NTP").style(Style::new().yellow()),
        Cell::from(ntp).style(Style::new().white()),
    ])
    .height(ntp_row_height as u16);

    let mut table = vec![iface_type_row, ip_source_row, dns_row, gateway_row, ntp_row];

    match &iface.media {
        NetworkType::Ethernet => {}
        NetworkType::WiFi(wifi_status) => {
            // Row 4: SSID
            let ssid = wifi_status
                .ssid
                .as_ref()
                .map_or("N/A".to_string(), |v| v.clone());
            let ssid_row = Row::new(vec![
                Cell::from("SSID").style(Style::new().yellow()),
                Cell::from(ssid).style(Style::new().white()),
            ]);
            table.push(ssid_row);
        }
        NetworkType::Cellular(_) => {}
    }

    table
}

impl IPresenter for NetworkPage {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        let estimated_width =
            IFACE_LABEL_LENGTH + LINK_STATE_LENGTH + IPV6_AVERAGE_LENGTH + MAC_LENGTH + 3 + 2 + 2; // for spacers and borders and selector
        let [dpc_info_rect, iface_list_rect, details_rect] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Fill(1),
        ])
        .areas(*area);
        let [list_rect, _unused_rect] =
            Layout::horizontal([Constraint::Length(estimated_width), Constraint::Fill(1)])
                .areas(iface_list_rect);

        self.render_dpc_info(model, dpc_info_rect, frame);
        self.render_interface_list(model, list_rect, frame);
        self.render_interface_details(model, details_rect, frame);
    }
}

impl NetworkPage {
    fn get_selected_interface(
        &self,
        model: &Rc<RefCell<MonitorModel>>,
    ) -> Option<NetworkInterfaceStatus> {
        let selected = self.selected()?;
        let model_ref = model.borrow();
        let ifaces = &model_ref.network;
        ifaces.iter().find(|iface| iface.name == selected).cloned()
    }

    fn render_interface_details(&mut self, model: &Rc<Model>, rect: Rect, frame: &mut Frame) {
        let iface = self.get_selected_interface(model);
        if iface.is_none() {
            return;
        }
        let iface = iface.unwrap();
        // create a table with the interface details. First column is the label, second column is the value
        // create header for the table
        let rows = details_table_from_iface(&iface);
        let table = Table::new(rows, [Constraint::Length(10), Constraint::Percentage(90)])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} Details", iface.name)),
            )
            .style(Style::default().fg(Color::White))
            .column_spacing(1);

        frame.render_widget(table, rect);
    }
    fn render_interface_list(&mut self, model: &Rc<Model>, list_rect: Rect, frame: &mut Frame) {
        // create header for the table
        let header = Row::new(vec![
            Cell::from("Name").style(Style::default()),
            Cell::from("Link").style(Style::default()),
            Cell::from("IPv4/IPv6").style(Style::default()),
            Cell::from("MAC").style(Style::default()),
        ]);

        // create list items from the interface
        let rows = model
            .borrow()
            .network
            .iter()
            .map(|iface| info_row_from_iface(iface))
            .collect::<Vec<_>>();

        self.list.size = rows.len();
        self.interface_names = model
            .borrow()
            .network
            .iter()
            .map(|iface| iface.name.clone())
            .collect();

        // create a surrounding block for the list
        let block = Block::default()
            .title(" Network Interfaces ")
            .title_alignment(Alignment::Center)
            .borders(Borders::TOP)
            .border_type(BorderType::Plain)
            // .border_style(Style::default().fg(Color::White).bg(Color::Black))
            // .style(Style::default().bg(Color::Black));
            .padding(Padding::new(1, 1, 1, 1));

        let bar = " █ ";

        // Create a List from all list items and highlight the currently selected one
        let list = Table::new(
            rows,
            [
                Constraint::Max(IFACE_LABEL_LENGTH),
                Constraint::Max(LINK_STATE_LENGTH),
                Constraint::Fill(1),
                Constraint::Max(MAC_LENGTH),
            ],
        )
        .block(block)
        .row_highlight_style(Style::new().bg(Color::DarkGray))
        // .highlight_symbol(">")
        .highlight_symbol(Text::from(vec![
            // "".into(),
            bar.into(),
            bar.into(),
            bar.into(),
            bar.into(),
            // "".into(),
        ]))
        .highlight_spacing(HighlightSpacing::Always)
        .header(header);

        StatefulWidget::render(list, list_rect, frame.buffer_mut(), &mut self.list.state);
    }

    fn render_dpc_info(&mut self, model: &Rc<Model>, rect: Rect, frame: &mut Frame) {
        let dpc_key = model.borrow().dpc_key.clone().unwrap_or("N/A".to_string());

        let configuration_string = match dpc_key.as_str() {
            "zedagent" => "From controller".green(),
            "manual" => "Set by local user".yellow(),
            s => s.red(),
        };

        // convert DPC key into human readabel piece of information
        let dpc_info = Line::default().spans(vec![
            "Current configuration: ".white(),
            configuration_string,
        ]);

        let mut text = Text::from(dpc_info);

        if dpc_key == "manual" {
            text.push_line(vec!["WARNING: ".red(),"the configuratiion set locally will be overwritten by working configuration from the controller".white()]);
        }

        // create paragraph with the DPC key
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, rect);
    }
}

impl IEventHandler for NetworkPage {
    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Home if key.modifiers == KeyModifiers::CONTROL => self.select_first(),
                KeyCode::End if key.modifiers == KeyModifiers::CONTROL => self.select_last(),
                KeyCode::Enter => {
                    let _selected_iface = self.selected();
                    if let Some(selected) = _selected_iface {
                        return Some(Action::new("net", UiActions::EditIfaceConfig(selected)));
                    }
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}

impl ISelector for NetworkPage {
    fn select_next(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            if selected < self.list.size - 1 {
                self.list.state.select(Some(selected + 1));
            }
        } else {
            self.list.state.select(Some(0));
        }
    }

    fn select_previous(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let index = selected.saturating_sub(1);
            self.list.state.select(Some(index));
        }
    }

    fn select_first(&mut self) {
        self.list.state.select(Some(0));
    }

    fn select_last(&mut self) {
        let index = self.list.size.saturating_sub(1);

        self.list.state.select(Some(index));
    }

    fn selected(&self) -> Option<String> {
        self.list
            .state
            .selected()
            .map(|index| self.interface_names[index].clone())
    }
}

pub fn create_network_page() -> impl IWindow {
    NetworkPage {
        list: InterfaceList::default(),
        interface_names: vec![],
    }
}
