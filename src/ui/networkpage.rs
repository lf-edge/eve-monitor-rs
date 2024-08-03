use core::fmt;
use std::{fmt::Display, net::IpAddr, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, ListItem, Padding, Row, StatefulWidget,
        Table, TableState,
    },
    Frame,
};

use crate::{
    device::network::NetworkInterface,
    events::Event,
    model::Model,
    traits::{IEventHandler, IPresenter, IWindow},
};

use super::action::Action;

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
            state: TableState::default().with_selected(0),
            size: 0,
        }
    }
}

impl IWindow for NetworkPage {}

impl From<&NetworkInterface> for ListItem<'_> {
    fn from(iface: &NetworkInterface) -> Self {
        let mut spans = vec![
            Span::raw(format!("{:<10}", iface.name)),
            if iface.up {
                Span::styled(format!("{:5}", "UP"), Style::new().green())
            } else {
                Span::styled(format!("{:5}", "DOWN"), Style::new().red())
            },
        ];

        // collect IP addresses and add as multiline
        if let Some(ips) = &iface.ipv4 {
            for ip in ips {
                spans.push(Span::styled(ip.to_string(), Style::new().blue()));
            }
        }

        let line = Line::from(spans);
        ListItem::new(line)
    }
}

impl From<&NetworkInterface> for Row<'_> {
    fn from(iface: &NetworkInterface) -> Self {
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

        // join both ipv4 and ipv6 addresses and separate by newline
        let combined_ip_list_iter = iface
            .ipv4
            .iter()
            .chain(iface.ipv6.iter())
            .flat_map(|v| v.iter().cloned())
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        // cell #3 IP address list
        if height > 1 {
            cells.push(Cell::from(combined_ip_list_iter).style(Style::new().white()));
        } else {
            cells.push(Cell::from("N/A").style(Style::new().red()));
        }

        // cell #4 MAC
        cells.push(Cell::from(iface.mac.to_string()).style(Style::new().yellow()));

        Row::new(cells).height(height as u16)
    }
}

impl IPresenter for NetworkPage {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, model: &Rc<Model>, _focused: bool) {
        let [list_rect, _details_rect] =
            Layout::horizontal([Constraint::Length(70), Constraint::Fill(1)]).areas(*area);

        // create header for the table
        let header = Row::new(vec![
            Cell::from("Name").style(Style::new().bold()),
            Cell::from("Link").style(Style::new().bold()),
            Cell::from("IPv4/Ipv6").style(Style::new().bold()),
            Cell::from("MAC").style(Style::new().bold()),
        ]);

        // create list items from the interface
        let rows = model
            .borrow()
            .network
            .iter()
            .map(|iface| Row::from(iface))
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
            .padding(Padding::new(1, 0, 1, 0));

        let bar = " █ ";

        // Create a List from all list items and highlight the currently selected one
        let list = Table::new(
            rows,
            [
                Constraint::Max(10),
                Constraint::Max(4),
                Constraint::Fill(1),
                Constraint::Max(16),
            ],
        )
        .block(block)
        .highlight_style(Style::new().bg(Color::DarkGray))
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
                    //TODO: send action to show dialog to edit interface
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}

trait ISelector {
    fn select_next(&mut self);
    fn select_previous(&mut self);
    fn select_first(&mut self);
    fn select_last(&mut self);
    fn selected(&self) -> Option<String>;
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
