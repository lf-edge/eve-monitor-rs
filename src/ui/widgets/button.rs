use crossterm::event::{KeyCode, KeyEvent};
use log::{info, trace};
use ratatui::{
    buffer::Buffer,
    layout::Alignment,
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    events::Event,
    traits::{IEventHandler, IFocusAcceptor, IWidget},
};

use super::element::{Element, IStandardRenderer};
use ratatui::widgets::WidgetRef;

pub struct ButtonWidgetState {
    label: String,
    pushed: bool,
    _on_click: Option<OnButtonClicked>,
}

pub type OnButtonClicked = Box<dyn Fn(&ButtonElement) -> ()>;

pub type ButtonElement = Element<ButtonWidgetState>;

impl ButtonElement {
    pub fn new<S: Into<String>>(label: S) -> Self {
        let state = ButtonWidgetState {
            label: label.into(),
            pushed: false,
            _on_click: None,
        };
        Self {
            d: state,
            v: Default::default(),
        }
    }
    fn is_pushed(&self) -> bool {
        self.d.pushed
    }
}

impl IStandardRenderer for ButtonElement {
    fn render(&self, area: &Rect, buf: &mut Buffer) {
        trace!(
            "Rendering button: {:?}: focused: {}",
            self.d.label.as_str(),
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
            Paragraph::new(self.d.label.as_str())
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .alignment(Alignment::Center)
                .block(block)
        } else {
            Paragraph::new(self.d.label.as_str())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .block(block)
        };
        button.render_ref(*area, buf);
    }
}

impl IEventHandler for ButtonElement {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<crate::events::Event> {
        info!("Handling key event: {:?}", key);
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    self.d.pushed = true;
                    info!("Button pushed");
                    return Some(Event::redraw());
                // TODO: Release event never comes if crossterm::event::PushKeyboardEnhancementFlags
                // is not enabled.
                } else if key.kind == crossterm::event::KeyEventKind::Release {
                    info!("Button released");
                    self.d.pushed = false;
                    return Some(Event::redraw());
                } else {
                    return None;
                }
            }
            _ => None,
        }
    }
}

impl IWidget for ButtonElement {}
