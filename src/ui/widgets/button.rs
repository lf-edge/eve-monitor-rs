use crossterm::event::{KeyCode, KeyEvent};
use log::{info, trace};
use ratatui::{
    layout::Alignment,
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    traits::{IElementEventHandler, IFocusAcceptor, IWidget, IWidgetPresenter},
    ui::action::UiActions,
};

use super::element::VisualState;
use ratatui::widgets::WidgetRef;

//pub type ButtonElement<A> = Element<ButtonWidgetState<A>>;
pub struct ButtonElement {
    v: VisualState,
    label: String,
    pushed: bool,
}

impl ButtonElement {
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            label: label.into(),
            pushed: false,
            v: Default::default(),
        }
    }
    fn is_pushed(&self) -> bool {
        self.pushed
    }
}

impl IWidgetPresenter for ButtonElement {
    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        trace!(
            "Rendering button: {:?}: focused: {}",
            self.label.as_str(),
            self.has_focus()
        );
        // set border style based on focus
        let border_style = if self.has_focus() {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        // set border type based on push state
        let border_type = if self.has_focus() {
            BorderType::Thick
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .style(Style::default().bg(Color::Black));

        let button = if self.is_pushed() {
            Paragraph::new(self.label.as_str())
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        } else {
            Paragraph::new(self.label.as_str())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .block(block)
        };
        button.render_ref(*area, frame.buffer_mut());
    }
}

impl IElementEventHandler for ButtonElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<UiActions> {
        info!("Handling key event: {:?}", key);
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    self.pushed = true;
                    info!("Button pushed");

                    return UiActions::ButtonClicked(self.label.clone()).into();

                // TODO: Release event never comes if crossterm::event::PushKeyboardEnhancementFlags
                // is not enabled.
                } else if key.kind == crossterm::event::KeyEventKind::Release {
                    info!("Button released");
                    self.pushed = false;
                    return None;
                } else {
                    return None;
                }
            }
            _ => None,
        }
    }
}

impl IWidget for ButtonElement {}

impl IFocusAcceptor for ButtonElement {
    fn set_focus(&mut self) {
        self.v.focused = true;
    }

    fn clear_focus(&mut self) {
        self.v.focused = false;
    }

    fn has_focus(&self) -> bool {
        self.v.focused
    }

    fn can_focus(&self) -> bool {
        self.v.can_focus
    }
}
