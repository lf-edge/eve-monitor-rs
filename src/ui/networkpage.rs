use std::rc::Rc;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, WidgetRef},
    Frame,
};

use crate::{model::Model, traits::IWindow};

use super::{tools::centered_rect, widgets::label::LabelElement, window::Window};

#[derive(Default)]
struct NetworkPageState {
    // we can save intermediate results here
    num_relayout: u32,
    initialized: bool,
}

fn network_page_layout(w: &mut Window<NetworkPageState>, rect: &Rect, _model: &Rc<Model>) {
    w.state.num_relayout += 1;
    // EXAMPLE: state or model changed
    if !w.state.initialized {
        let clock = LabelElement::new("Clock").on_tick(|label| {
            let now = chrono::Local::now();
            let time = now.format("%H:%M:%S").to_string();
            label.set_text(time);
        });
        w.add_widget("Clock 1", Box::new(clock));
    }
    w.state.initialized = true;

    // Update layout regardless...
    let rect = centered_rect(40, 10, *rect);
    w.update_layout("Clock 1", rect);
    // Custom layout
    w.update_layout("CustomFrame", rect)
}

fn network_page_render(
    w: &mut Window<NetworkPageState>,
    _rect: &Rect,
    frame: &mut Frame<'_>,
    _model: &Rc<Model>,
) {
    //custom frame
    let rect = w.layout("CustomFrame");

    let blk = Block::new()
        //.border_type(BorderType::Rounded)
        //FIXME: need new Font
        .border_type(BorderType::Plain)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black))
        .title(format!("Num Relayout: {}", w.state.num_relayout));
    blk.render_ref(rect, frame.buffer_mut());

    // render custom temp widget
}

pub fn create_network_page() -> impl IWindow {
    let window = Window::<NetworkPageState>::builder("Network Page")
        .with_state(NetworkPageState::default())
        .with_layout(network_page_layout)
        .with_render(network_page_render)
        .build();
    window.unwrap()
}
