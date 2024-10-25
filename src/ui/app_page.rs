use std::rc::Rc;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Padding, Row, StatefulWidget, Table,
        TableState,
    },
    Frame,
};

use crate::{
    events::Event,
    model::model::{AppInstance, AppInstanceState, Model},
    traits::{IEventHandler, IPresenter, IWindow},
};

use super::traits::ISelector;

#[derive(Debug, Default)]
struct ApplicationList {
    state: TableState,
    size: usize,
}

#[derive(Debug, Default)]
pub struct ApplicationsPage {
    list: ApplicationList,
}

impl ApplicationsPage {
    pub fn new() -> Self {
        ApplicationsPage {
            ..Default::default()
        }
    }
    fn render_app_list(&mut self, model: &Rc<Model>, list_rect: Rect, frame: &mut Frame) {
        // create header for the table
        let header = Row::new(vec![
            Cell::from("Name").style(Style::default()),
            Cell::from("GUID").style(Style::default()),
            Cell::from("Status").style(Style::default()),
        ]);

        // create list items from the interface
        let rows = model
            .borrow()
            .apps
            .iter()
            .map(|(_, app)| info_row_from_app(app))
            .collect::<Vec<_>>();

        self.list.size = rows.len();
        // self.interface_names = model
        //     .borrow()
        //     .network
        //     .iter()
        //     .map(|iface| iface.name.clone())
        //     .collect();

        // create a surrounding block for the list
        let block = Block::default()
            .title(" Applications ")
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
                Constraint::Max(20),
                Constraint::Max(32),
                Constraint::Fill(14),
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

impl IWindow for ApplicationsPage {}

impl IEventHandler for ApplicationsPage {
    fn handle_event(&mut self, event: Event) -> Option<super::action::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Home if key.modifiers == KeyModifiers::CONTROL => self.select_first(),
                KeyCode::End if key.modifiers == KeyModifiers::CONTROL => self.select_last(),
                KeyCode::Enter => {
                    let _selected_iface = self.selected();
                    // if let Some(selected) = _selected_iface {
                    //     return Some(Action::new("net", UiActions::EditIfaceConfig(selected)));
                    // }
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
}

fn info_row_from_app<'a, 'b>(app: &'a AppInstance) -> Row<'b> {
    let height = 1;
    // cells #1,2 IFace name and Link status
    let mut cells = vec![
        Cell::from(app.name.clone()),
        Cell::from(app.uuid.to_string()),
        match &app.state {
            AppInstanceState::Normal(st) => Cell::from(st.to_string()).style(Style::new().green()),
            AppInstanceState::Error(st, _err) => {
                Cell::from(st.to_string()).style(Style::new().red())
            }
        },
    ];

    // // collect IP addresses and add as multiline
    // let ipv4_len = app.ipv4.as_ref().map_or(0, |v| v.len());
    // let ipv6_len = app.ipv6.as_ref().map_or(0, |v| v.len());

    // let height = (ipv4_len + ipv6_len).max(1);

    // // join both ipv4 and ipv6 addresses and separate by newline
    // let combined_ip_list_iter = app
    //     .ipv4
    //     .iter()
    //     .chain(app.ipv6.iter())
    //     .flat_map(|v| v.iter().cloned())
    //     .map(|ip| ip.to_string())
    //     .collect::<Vec<_>>()
    //     .join("\n");

    // // cell #3 IP address list
    // if height > 1 {
    //     cells.push(Cell::from(combined_ip_list_iter).style(Style::new().white()));
    // } else {
    //     cells.push(Cell::from("N/A").style(Style::new().red()));
    // }

    // cell #4 MAC
    //cells.push(Cell::from(app.mac.to_string()).style(Style::new().yellow()));

    Row::new(cells).height(height)
}

impl IPresenter for ApplicationsPage {
    fn render(
        &mut self,
        area: &ratatui::prelude::Rect,
        frame: &mut ratatui::Frame<'_>,
        model: &std::rc::Rc<Model>,
        focused: bool,
    ) {
        self.render_app_list(model, *area, frame);
    }
}

impl ISelector for ApplicationsPage {
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
        // self.list
        //     .state
        //     .selected()
        //     .map(|index| self[index].clone())
        None
    }
}
