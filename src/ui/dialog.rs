use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use crate::traits::Component;

use super::{button::Button, label::Label};

struct Dialog {
    title: String,
    content: Vec<Box<dyn Component>>,
    //buttons: Vec<String>,
    buttons: Vec<Box<Button>>,
    size: (u16, u16),
}

impl Dialog {
    fn new(title: String, buttons: Vec<String>, size: (u16, u16)) -> Self {
        let buttons = buttons
            .into_iter()
            .map(|b| Box::new(Button::new(b)))
            .collect();
        Self {
            title,
            content: Vec::new(),
            buttons,
            size,
        }
    }
    fn set_content(&mut self, content: Vec<Box<dyn Component>>) {
        self.content = content;
    }
}

impl Component for Dialog {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        // get centered area for the dialog
        let area = centered_rect(self.size.0, self.size.1, *area);

        // create a block with wide borders
        let blk = Block::new()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.title.as_str());

        let inner_area = blk.inner(area);
        // render the block
        frame.render_widget(blk, area);

        let layout =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner_area);
        let layout_buttons = Layout::horizontal(
            self.buttons
                .iter()
                .map(|b| Constraint::Length(b.label().len() as u16 + 4))
                .collect::<Vec<_>>(),
        )
        .flex(ratatui::layout::Flex::End)
        .split(layout[1]);
        // render buttons
        // for (i, button) in self.buttons.iter().enumerate() {
        //     // render the block around buttons
        //     let blk = Block::default()
        //         .borders(Borders::ALL)
        //         .border_type(BorderType::Rounded)
        //         .border_style(Style::default().fg(Color::White))
        //         .style(Style::default().bg(Color::Black))
        //         .padding(Padding::horizontal(1));
        //     let text_area = blk.inner(layout_buttons[i]);

        //     frame.render_widget(blk, layout_buttons[i]);
        //     // render the button
        //     let button = Paragraph::new(button.as_str())
        //         .style(Style::default().fg(Color::Red))
        //         .alignment(Alignment::Center);
        //     frame.render_widget(button, text_area);
        // }
        for (i, button) in self.buttons.iter_mut().enumerate() {
            button.render(&layout_buttons[i], frame);
        }

        // render content
        for c in self.content.iter_mut() {
            c.render(&layout[0], frame);
        }

        //frame.render_widget(dialog, *area);
    }

    fn handle_event(&mut self, _event: &crate::events::Event) -> Option<crate::events::Event> {
        for button in self.buttons.iter_mut() {
            if let Some(r) = button.handle_event(_event) {
                return Some(r);
            }
        }
        None
    }
}

pub fn message_box(title: &str, content: &str, buttons: Vec<String>) -> impl Component {
    let mut dlg = Dialog::new(title.to_string(), buttons, (30, 15));
    let content = Label::new(content.to_string());
    dlg.set_content(vec![Box::new(content)]);
    dlg
}

// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
