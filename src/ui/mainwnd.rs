// impl VisualComponent for MainWnd {
//     fn render(&mut self, area: &Rect, frame: &mut Frame<'_>, focused: bool) {
//         use Constraint::{Length, Min};
//         let [main_area, status_bar_area] = Layout::vertical([Min(0), Length(3)]).areas(*area);
//         self.input_field.render(&main_area, frame, focused);
//         self.status_bar.render(&status_bar_area, frame, focused);
//     }
// }

use std::{collections::HashMap, fmt::Debug};

use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    dispatcher::EventDispatcher,
    events::UiCommand,
    traits::{
        IEventDispatcher, IEventHandler, IFocusAcceptor, IFocusTracker, IPresenter, IVisible,
        IVisibleElement, IWindow,
    },
};

use super::focus_tracker::{FocusMode, FocusTracker};

pub trait IWidget: IPresenter + IEventHandler {}

struct WindowBuilder {}
pub struct MainWnd {
    //status_bar: Box<StatusBarWidget>,
    //id: WindowId,
    pub ft: FocusTracker,
    //pub rg: RadioGroupView,
    pub widgets: HashMap<String, Box<dyn IWidget>>,
    pub layout: HashMap<String, Rect>,
}

impl Debug for MainWnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MainWnd");
        // print the layout
        for (k, v) in &self.layout {
            write!(f, "layout: {} => {:#?}", k, v);
        }
        // print focus tracker
        write!(f, "focus tracker: {:#?}", &self.ft);
        Ok(())
    }
}

impl MainWnd {
    // fn add_radio_group(&mut self, labels: Vec<String>, title: String) {
    //     let mut rg = RadioGroupState {
    //         labels: labels.clone(),
    //         selected: 0,
    //         title,
    //     };
    //     let widget = RadioGroupWidget {};
    //     self.rg = RadioGroupView {
    //         state: rg,
    //         widget,
    //         ft: FocusTracker::create_from_taborder(labels, None, FocusMode::Wrap),
    //     };
    // }
    fn add_widget<W: StatefulWidgetRef>(&mut self, widget: W) {
        todo!()
    }
}

impl IWindow for MainWnd {}
impl IEventDispatcher for MainWnd {
    fn dispatch_event(&self, event: UiCommand) {
        todo!()
    }
}
impl IEventHandler for MainWnd {
    fn handle_key_event(&mut self, key: KeyEvent) {
        // forward the event to the focused view
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(focused_view).unwrap();
            widget.handle_key_event(key);
        }
    }
}

impl IFocusTracker for MainWnd {
    fn focus_next(&mut self) -> Option<&String> {
        info!("focus_next: MainWnd {:#?}", &self.ft);
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(focused_view).unwrap();
            widget.clear_focus();
        }
        let next = self.ft.focus_next();
        if let Some(focused_view) = next {
            let widget = self.widgets.get_mut(focused_view).unwrap();
            widget.set_focus();
        }
        next
    }

    fn focus_prev(&mut self) -> Option<&String> {
        info!("focus_prev: MainWnd {:#?}", &self.ft);
        if let Some(focused_view) = self.ft.get_focused_view() {
            let widget = self.widgets.get_mut(focused_view).unwrap();
            widget.clear_focus();
        }
        let next = self.ft.focus_prev();
        if let Some(focused_view) = next {
            let widget = self.widgets.get_mut(focused_view).unwrap();
            widget.set_focus();
        }
        next
    }

    fn get_focused_view_name(&self) -> Option<&String> {
        self.ft.get_focused_view()
    }
}
#[derive(Debug)]
pub struct RadioGroupState {
    pub labels: Vec<String>,
    pub selected: usize,
    pub title: String,
    pub in_focus: bool,
    pub is_visible: bool,
}

trait ILayout {
    fn get_layout(&self) -> HashMap<String, ratatui::prelude::Rect>;
    fn set_layout(&self, layout: HashMap<String, ratatui::prelude::Rect>);
}

impl ILayout for RadioGroupState {
    fn get_layout(&self) -> HashMap<String, ratatui::prelude::Rect> {
        todo!()
    }

    fn set_layout(&self, layout: HashMap<String, ratatui::prelude::Rect>) {
        todo!()
    }
}

impl IFocusAcceptor for RadioGroupState {
    fn set_focus(&mut self) {
        self.in_focus = true;
    }

    fn clear_focus(&mut self) {
        self.in_focus = false;
    }
}

impl IVisible for RadioGroupState {
    fn is_visible(&self) -> bool {
        self.is_visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
}

#[derive(Debug)]
pub struct RadioGroupWidget {}

impl StatefulWidgetRef for RadioGroupWidget {
    type State = RadioGroupState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        //let layout = state.get_layout();
        info!("rendering: RadioGroupWidget {:#?}", &state);
        let style = if state.in_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(state.title.clone())
            .borders(Borders::ALL)
            .border_style(style);
        let inner = block.inner(area);
        block.render_ref(area, buf);
        // create vertical layout for radio buttons
        let constraints = state.labels.iter().map(|_| Constraint::Length(1));
        let buttons_area = Layout::vertical(constraints).split(inner);

        // render paragraphs for each radio button
        for (i, label) in state.labels.iter().enumerate() {
            // format the button label <text> (selected)
            let label = if state.selected == i {
                format!("{} (*)", label)
            } else {
                format!("{} ( )", label)
            };

            let p = Paragraph::new(label);
            p.render_ref(buttons_area[i], buf);
        }
    }
}
#[derive(Debug)]
pub struct WidgetWithLayout<W> {
    widget: W,
    layout: HashMap<String, Rect>,
}

impl<W> WidgetWithLayout<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            layout: HashMap::new(),
        }
    }
}

impl<W> StatefulWidgetRef for WidgetWithLayout<W>
where
    W: StatefulWidgetRef,
    W::State: ILayout,
{
    type State = W::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Now you can access ILayout methods on the state
        //state.get_layout(); //

        // Delegate to the original widget's render_ref method
        self.widget.render_ref(area, buf, state);
    }
}

impl<W> StatefulWidgetRef for &mut WidgetWithLayout<W>
where
    W: StatefulWidgetRef,
    W::State: ILayout + IVisible + IFocusAcceptor,
{
    type State = W::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Now you can access ILayout methods on the state
        //state.get_layout();

        // Delegate to the original widget's render_ref method
        self.widget.render_ref(area, buf, state);
    }
}

#[derive(Debug)]
pub struct RadioGroupView {
    pub state: RadioGroupState,
    pub widget: WidgetWithLayout<RadioGroupWidget>,
    pub ft: FocusTracker,
}

impl IFocusAcceptor for RadioGroupView {
    fn set_focus(&mut self) {
        self.state.set_focus();
    }

    fn clear_focus(&mut self) {
        self.state.clear_focus();
    }
}

impl IVisible for RadioGroupView {
    fn is_visible(&self) -> bool {
        self.state.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.state.set_visible(visible);
    }
}

impl IPresenter for RadioGroupView {
    fn do_layout(
        &mut self,
        area: &Rect,
    ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
        // let mut layout_map = std::collections::HashMap::new();
        // layout_map.insert("RadioGroup".to_string(), *area);

        // info!("do_layout: RadioGroupView {:#?}", &self);
        // return HashMap::new();
        todo!("RadioGroupView::do_layout")
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        info!("rendering: RadioGroupView {:#?}", &self);
        frame.render_stateful_widget_ref(&mut self.widget, *area, &mut self.state);
        // self.widget
        //     .render_ref(*area, frame.buffer_mut(), &mut self.state)
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}

impl IFocusTracker for RadioGroupView {
    fn focus_next(&mut self) -> Option<&String> {
        self.ft.focus_next()
    }

    fn focus_prev(&mut self) -> Option<&String> {
        self.ft.focus_prev()
    }

    fn get_focused_view_name(&self) -> Option<&String> {
        Some(&self.state.labels[self.state.selected])
    }
}

impl IEventHandler for RadioGroupView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        info!("handle_key_event: RadioGroupView {:#?}", &self);
        //TODO: change to focus tracker
        match key.code {
            KeyCode::Up => {
                self.state.selected = self.state.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.state.selected = (self.state.selected + 1).min(self.state.labels.len() - 1);
            }
            _ => {}
        }
    }
}
impl IWidget for RadioGroupView {}

impl IVisible for MainWnd {}
impl IFocusAcceptor for MainWnd {}
impl IPresenter for MainWnd {
    fn do_layout(
        &mut self,
        area: &Rect,
    ) -> std::collections::HashMap<String, ratatui::prelude::Rect> {
        let mut layout = HashMap::new();
        let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(*area);
        for (i, col) in cols.iter().enumerate() {
            let rows = Layout::vertical([Constraint::Ratio(1, 4); 4]).split(*col);
            for (j, row) in rows.iter().enumerate() {
                let area_name = format!("{}-{}", i, j);
                layout.insert(area_name, *row);
            }
        }
        self.layout = layout.clone();
        info!("do_layout: MainWnd {:#?}", &self.layout);
        layout
    }

    fn render(&mut self, area: &Rect, frame: &mut ratatui::Frame<'_>) {
        info!("rendering: MainWnd {:#?}", &self);
        let r = self.layout.get("0-0").unwrap();
        let rg = self.widgets.get_mut("RadioGroup").unwrap();
        rg.render(r, frame);

        let r = self.layout.get("0-1").unwrap();
        let rg = self.widgets.get_mut("RadioGroup 1").unwrap();

        rg.render(r, frame);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}
