use crate::actions::MonActions;
use crate::ui::action::UiActions;
use crate::ui::window::LayoutMap;
use crate::ui::window::Window;
use log::debug;
use log::info;
use log::warn;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::prelude::Rect;
use ratatui::Frame;

struct HomePage {}

#[derive(Clone)]
struct SummaryState {
    a: i32,
    ip: String,
}

impl HomePage {
    pub fn init(&self) -> Window<MonActions, SummaryState> {
        let wnd = Window::builder("MainWnd")
            .with_state(SummaryState {
                a: 42,
                ip: "10.208.13.5".to_string(),
            })
            // .widget("Button", Box::new(button))
            // .widget("Input", Box::new(input))
            .with_layout(HomePage::do_layout)
            .with_render(Self::do_render)
            .with_focused_view("Input")
            .on_action(|action, state: &mut SummaryState| {
                debug!("on_action Action: {:?}", action);
                match action.action {
                    UiActions::CheckBox { checked: _ } => todo!(),
                    UiActions::RadioGroup { selected: _ } => todo!(),
                    UiActions::Input { text } => {
                        info!("Input updated: {}", &text);
                        state.ip = text;
                    }

                    _ => {
                        warn!("Unhandled action: {:?}", action);
                    }
                }
                // match action.action {
                //     MonActions::ButtonClicked(label) => {
                //         state.a += 1;
                //         info!("Button clicked: {} counter {}", label, state.a);
                //         return Some(MonActions::MainWndStateUpdated(state.clone()));
                //     }
                //     MonActions::InputUpdated(input) => {
                //         info!("Input updated: {}", input);
                //         return Some(MonActions::MainWndStateUpdated(state.clone()));
                //     }
                //     _ => {}
                // }
                None
            })
            .build()
            .unwrap();

        wnd
    }

    pub fn do_layout(area: &Rect) -> Option<LayoutMap> {
        let chunks =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(*area);
        let [left, right] = [chunks[0], chunks[1]];

        let mut lm = LayoutMap::new();
        lm.add("left".to_string(), left.clone());
        lm.add("right".to_string(), right.clone());
        Some(lm)
    }

    pub fn do_render<T>(_area: &Rect, _frame: &mut Frame<'_>, _layout: &LayoutMap) {}

    fn draw(&self, _frame: &mut Frame, _area: Rect) {}
}
