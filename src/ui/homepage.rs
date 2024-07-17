use crate::traits::IPresenter;
use crate::ui::window::LayoutMap;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::prelude::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

struct HomePage {
    state: SummaryState,
    layout: Option<LayoutMap>,
}

#[derive(Clone, Debug)]
struct SummaryState {
    a: i32,
    ip: String,
}

impl HomePage {
    pub fn new() -> Self {
        let hp = HomePage {
            layout: None,
            state: SummaryState {
                a: 1,
                ip: "thing".to_string(),
            },
        };
        hp
    }
    // pub fn init(&self) -> Window<MonActions, SummaryState> {
    //     let wnd = Window::builder("MainWnd")
    //         .with_state(SummaryState {
    //             a: 42,
    //             ip: "10.208.13.5".to_string(),
    //         })
    //         // .widget("Button", Box::new(button))
    //         // .widget("Input", Box::new(input))
    //         .with_layout(HomePage::do_layout)
    //         .with_render(Self::do_render)
    //         .with_focused_view("Input")
    //         .on_action(|action, state: &mut SummaryState| {
    //             debug!("on_action Action: {:?}", action);
    //             match action.action {
    //                 UiActions::CheckBox { checked: _ } => todo!(),
    //                 UiActions::RadioGroup { selected: _ } => todo!(),
    //                 UiActions::Input { text } => {
    //                     info!("Input updated: {}", &text);
    //                     state.ip = text;
    //                 }

    //                 _ => {
    //                     warn!("Unhandled action: {:?}", action);
    //                 }
    //             }
    //             // match action.action {
    //             //     MonActions::ButtonClicked(label) => {
    //             //         state.a += 1;
    //             //         info!("Button clicked: {} counter {}", label, state.a);
    //             //         return Some(MonActions::MainWndStateUpdated(state.clone()));
    //             //     }
    //             //     MonActions::InputUpdated(input) => {
    //             //         info!("Input updated: {}", input);
    //             //         return Some(MonActions::MainWndStateUpdated(state.clone()));
    //             //     }
    //             //     _ => {}
    //             // }
    //             None
    //         })
    //         .build()
    //         .unwrap();

    //     wnd
    // }

    pub fn do_layout(&self, area: &Rect) -> LayoutMap {
        let chunks =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(*area);
        let [left, right] = [chunks[0], chunks[1]];

        let mut lm = LayoutMap::new();
        lm.add("left".to_string(), left.clone());
        lm.add("right".to_string(), right.clone());
        lm
    }

    pub fn do_render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        if self.layout.is_none() {
            self.layout = Some(self.do_layout(area));
        }
        let layout = self.layout.as_ref().unwrap();

        let left = Paragraph::new(format!("{0:?}", self.state));
        frame.render_widget(left, layout["left"]);
    }

    fn draw(&self, _frame: &mut Frame, _area: Rect) {}
}

impl IPresenter for HomePage {
    // add code here
    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        self.do_render(area, frame)
    }
    fn is_focus_tracker(&self) -> bool {
        false
    }
}
