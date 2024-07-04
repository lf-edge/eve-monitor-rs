use std::{any::Any, collections::HashMap};

use log::trace;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, StatefulWidgetRef},
    Frame,
};

use crate::traits::{Component, VisualComponent};

use super::{
    button::Button,
    component::{StatefulComponentWrapper, WidgetState},
    label::Label,
    // mainwnd::MainWnd,
    window::{Window, WindowId},
};

pub fn create_dialog(title: &str, buttons: Vec<String>) -> Window {
    let mut root_view = Dialog::new(title.to_string(), (30, 15));
    let content = Label::new("This is a dialog".to_string());
    root_view.set_content(&mut vec![Box::new(content)]);
    buttons
        .into_iter()
        .map(|label| {
            let button = Button::new(label);
            Box::new(button)
        })
        .for_each(|e| {
            root_view.add_button(e);
        });
    Window::new(Box::new(root_view))
}

// pub fn create_main_window() -> Window {
//     let root_view = MainWnd::new();
//     Window::new(Box::new(root_view))
// }

// struct Dialog {
//     title: String,
//     content: Vec<Box<dyn Component>>,
//     buttons: Vec<Box<Button>>,
//     size: (u16, u16),
// }

struct DialogWidgetState {
    title: String,
    // content: Vec<Box<dyn VisualComponent>>,
    buttons: HashMap<String, WindowId>,
    size: (u16, u16),
}
impl WidgetState for DialogWidgetState {}
struct DialogWidget {}
impl StatefulWidgetRef for DialogWidget {
    type State = DialogWidgetState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        todo!()
    }
}

type Dialog = StatefulComponentWrapper<DialogWidget, DialogWidgetState>;

impl Dialog {
    fn new(title: String, size: (u16, u16)) -> Self {
        Self::create_component_state(
            Box::new(DialogWidget {}),
            Box::new(DialogWidgetState {
                title,
                size,
                // content: Vec::new(),
                buttons: HashMap::new(),
            }),
        )
    }
    fn add_button(&mut self, button: Box<Button>) {
        self.state
            .buttons
            .insert(button.state.label().to_string(), button.id());
        // prepend to the content
        self.root.insert(0, button);
    }
    fn set_content(&mut self, content: &mut Vec<Box<dyn VisualComponent>>) {
        self.root.append(content);
    }
}

impl VisualComponent for Dialog {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, _focused: bool) {
        // get centered area for the dialog
        let area = centered_rect(self.state.size.0, self.state.size.1, *area);

        // create a block with wide borders
        let blk = Block::new()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.state.title.as_str());

        let inner_area = blk.inner(area);
        // render the block
        frame.render_widget(blk, area);

        let layout =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner_area);
        let layout_buttons = Layout::horizontal(
            self.state
                .buttons
                .iter()
                .map(|(label, _)| Constraint::Length(label.len() as u16 + 4))
                .collect::<Vec<_>>(),
        )
        .flex(ratatui::layout::Flex::End)
        .split(layout[1]);

        let is_button = |c: &Box<dyn VisualComponent>| -> bool {
            self.state.buttons.values().any(|id| *id == c.id())
        };
        // render buttons
        for (i, button) in self.root.iter_mut().filter(|c| is_button(c)).enumerate() {
            button.render(&layout_buttons[i], frame, _focused);
        }
        // render content
        for c in self.root.iter_mut().filter(|c| !is_button(c)) {
            c.render(&layout[0], frame, _focused);
        }
    }

    fn handle_event(&mut self, _event: &crate::events::Event) -> Option<crate::events::Event> {
        // for button in self.buttons.iter_mut() {
        //     if let Some(r) = button.handle_event(_event) {
        //         return Some(r);
        //     }
        // }
        None
    }

    // fn id(&self) -> super::window::WindowId {
    //     self.id
    // }

    // fn can_focus(&self) -> bool {
    //     trace!("Dialog can_focus");
    //     true
    // }

    // fn focus_next(&mut self) -> Option<WindowId> {
    //     None
    // }

    // fn focus_prev(&mut self) -> Option<WindowId> {
    //     None
    // }

    // fn focus_lost(&mut self) {
    //     trace!("Dialog focus_lost");
    //     for b in self.buttons.iter_mut() {
    //         b.focus_lost();
    //     }
    // }

    // fn focus_gain(&mut self) -> Option<WindowId> {
    //     trace!("Dialog focus_gain");
    //     // set focus to 'cancel' button by default
    //     if let Some(b) = self.buttons.iter_mut().find(|b| b.label() == "Cancel") {
    //         return b.focus_gain();
    //     }
    //     None
    // }

    // fn get_children(&self) -> Vec<(WindowId, WindowId)> {
    //     let buttons = self.buttons.iter().map(|b| (self.id, b.id()));
    //     let content = self.content.iter().map(|c| c.get_children()).flatten();
    //     buttons.chain(content).collect()
    // }
}

pub fn message_box(title: &str, content: &str, buttons: Vec<String>) -> impl Component {
    let mut dlg = Dialog::new(title.to_string(), (30, 15));
    let content = Label::new(content.to_string());
    dlg.set_content(&mut vec![Box::new(content)]);

    buttons
        .into_iter()
        .map(|label| {
            let button = Button::new(label);
            Box::new(button)
        })
        .for_each(|e| {
            dlg.add_button(e);
        });

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
