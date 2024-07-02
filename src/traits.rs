use ratatui::{layout::Rect, widgets::Widget, Frame};

use crate::events::Event;

pub trait Component {
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>);
    fn handle_event(&mut self, _event: &Event) -> Option<Event> {
        None
    }
}

// impl Component for Box<dyn Component> {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
//         self.as_mut().render(area, frame);
//     }
//     fn handle_event(&mut self, event: &Event) -> Option<Event> {
//         self.as_mut().handle_event(event)
//     }
// }

// impl Component for &mut dyn Component {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
//         self.render(area, frame);
//     }
//     fn handle_event(&mut self, event: Event) -> Option<Event> {
//         self.handle_event(event)
//     }
// }

// impl Component for &mut Box<dyn Component> {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
//         self.as_mut().render(area, frame);
//     }
//     fn handle_event(&mut self, event: &Event) -> Option<Event> {
//         self.as_mut().handle_event(event)
//     }
// }

// impl<T: Widget> Component for T {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
//         frame.render_widget(&self, *area);
//     }
// }

// implement Debug for dyn traits::Component + 'static
impl std::fmt::Debug for dyn Component + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component")
    }
}
