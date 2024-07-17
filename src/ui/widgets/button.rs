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

pub type OnButtonClicked<A> = dyn FnMut(&String) -> Option<UiActions<A>>;

//pub type ButtonElement<A> = Element<ButtonWidgetState<A>>;
pub struct ButtonElement<A> {
    v: VisualState,
    label: String,
    pushed: bool,
    on_click: Option<Box<OnButtonClicked<A>>>,
}

impl<A> ButtonElement<A> {
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            label: label.into(),
            pushed: false,
            on_click: None,
            v: Default::default(),
        }
    }
    pub fn on_click(mut self, f: Box<OnButtonClicked<A>>) -> Self {
        self.on_click = Some(f);
        self
    }
    fn is_pushed(&self) -> bool {
        self.pushed
    }
}

impl<'a, A> IWidgetPresenter for ButtonElement<A> {
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

impl<'a, A> IElementEventHandler for ButtonElement<A> {
    type Action = A;
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<UiActions<Self::Action>> {
        info!("Handling key event: {:?}", key);
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    self.pushed = true;
                    info!("Button pushed");
                    if let Some(f) = self.on_click.as_deref_mut() {
                        let custom_action = (f)(&self.label);
                        if let Some(action) = custom_action {
                            return Some(action);
                        }
                    } else {
                        return UiActions::ButtonClicked(self.label.clone()).into();
                    }
                    return None;
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

impl<A> IWidget for ButtonElement<A> {}

impl<A> IFocusAcceptor for ButtonElement<A> {
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
