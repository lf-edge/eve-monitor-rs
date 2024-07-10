use super::window::WidgetMap;

// use crate::traits::ViewComposer;
#[derive(Debug)]
pub enum FocusMode {
    Wrap,
    OneShot,
}
#[derive(Debug)]
pub struct FocusTracker {
    focused_view: usize,
    tab_order: Vec<String>,
    focus_mode: FocusMode,
    too_late: bool,
}

impl FocusTracker {
    pub fn new(
        tab_order: Vec<String>,
        focused_view: Option<String>,
        focus_mode: FocusMode,
    ) -> Self {
        // if focused view is set find its index in the tab order
        let focused_view = focused_view
            .and_then(|name| tab_order.iter().position(|n| n == &name))
            .unwrap_or(0);

        Self {
            focused_view,
            tab_order,
            focus_mode,
            too_late: false,
        }
    }

    pub fn create_from_taborder(
        tab_order: Vec<String>,
        focused_view: Option<String>,
        focus_mode: FocusMode,
    ) -> FocusTracker {
        let focus_tracker = FocusTracker::new(tab_order, focused_view, focus_mode);
        focus_tracker
    }

    pub fn create_from_views(
        views: &WidgetMap,
        focused_view: Option<String>,
        focus_mode: FocusMode,
    ) -> FocusTracker {
        let collect_views = || {
            let mut tab_order = Vec::new();

            for (view_name, view) in views.iter() {
                if view.is_focus_tracker() {
                    tab_order.push(view_name.clone());
                }
            }
            tab_order
        };

        let tab_order = collect_views();
        let focus_tracker = FocusTracker::new(tab_order, focused_view, focus_mode);
        focus_tracker
    }

    pub fn get_focused_view(&self) -> Option<String> {
        self.tab_order.get(self.focused_view).cloned()
    }
    pub fn focus_next(&mut self) -> Option<String> {
        if self.too_late {
            return None;
        }
        if self.focused_view + 1 < self.tab_order.len() {
            self.focused_view += 1;
        } else {
            if let FocusMode::Wrap = self.focus_mode {
                self.focused_view = 0;
            } else if let FocusMode::OneShot = self.focus_mode {
                self.too_late = true;
                return None;
            }
        }

        Some(self.tab_order[self.focused_view].clone())
    }

    pub fn focus_prev(&mut self) -> Option<String> {
        if self.too_late {
            return None;
        }
        if self.focused_view > 0 {
            self.focused_view -= 1;
        } else {
            if let FocusMode::Wrap = self.focus_mode {
                self.focused_view = self.tab_order.len() - 1;
            } else if let FocusMode::OneShot = self.focus_mode {
                self.too_late = true;
                return None;
            }
        }

        Some(self.tab_order[self.focused_view].clone())
    }
}
// #[cfg(test)]
// mod tests {
//     use crate::traits::{FocusTracker, View};

//     use super::*;

//     #[test]
//     fn test_focus_tracker() {
//         let tab_order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
//         let mut focus_tracker =
//             FocusTracker::create_from_taborder(tab_order.clone(), None, FocusMode::Wrap);

//         assert_eq!(focus_tracker.get_focused_view(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"b".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"b".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"a".to_string()));
//     }
//     #[test]
//     fn test_focus_tracker_one_shot() {
//         let tab_order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
//         let mut focus_tracker =
//             FocusTracker::create_from_taborder(tab_order.clone(), None, FocusMode::OneShot);

//         assert_eq!(focus_tracker.get_focused_view(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"b".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_next(), None); // No more views to focus
//         assert_eq!(focus_tracker.focus_prev(), None); // No more views to focus
//     }
//     #[test]
//     fn test_focus_tracker_with_focused_view() {
//         let tab_order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
//         let focused_view = Some("b".to_string());
//         let mut focus_tracker = FocusTracker::create_from_taborder(
//             tab_order.clone(),
//             focused_view.clone(),
//             FocusMode::Wrap,
//         );

//         assert_eq!(focus_tracker.get_focused_view(), focused_view.as_ref());
//         assert_eq!(focus_tracker.focus_next(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"b".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"c".to_string()));
//     }

//     #[test]
//     fn test_focus_tracker_one_shot_with_focused_view() {
//         let tab_order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
//         let focused_view = Some("b".to_string());
//         let mut focus_tracker = FocusTracker::create_from_taborder(
//             tab_order.clone(),
//             focused_view.clone(),
//             FocusMode::OneShot,
//         );

//         assert_eq!(focus_tracker.get_focused_view(), focused_view.as_ref());
//         assert_eq!(focus_tracker.focus_next(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_next(), None); // No more views to focus
//         assert_eq!(focus_tracker.focus_prev(), None); // No more views to focus
//     }
//     #[test]
//     fn test_focus_tracker_create_from_views() {
//         let mut views: HashMap<String, Box<dyn ViewComposer>> = HashMap::new();
//         views.insert("a".to_string(), Box::new(MockComponent::new(true)));
//         views.insert("b".to_string(), Box::new(MockComponent::new(false)));
//         views.insert("c".to_string(), Box::new(MockComponent::new(true)));

//         let mut focus_tracker = FocusTracker::create_from_views(&views, None, FocusMode::Wrap);

//         assert_eq!(focus_tracker.get_focused_view(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_next(), Some(&"a".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"c".to_string()));
//         assert_eq!(focus_tracker.focus_prev(), Some(&"a".to_string()));
//     }

//     struct MockComponent {
//         can_focus: bool,
//     }

//     impl MockComponent {
//         fn new(can_focus: bool) -> Self {
//             Self { can_focus }
//         }
//     }

//     impl View for MockComponent {
//         fn render(&self) -> String {
//             "MockComponent".to_string()
//         }
//         fn get_name(&self) -> &str {
//             "MockComponent"
//         }
//     }

//     impl FocusTracker for MockComponent {
//         fn can_focus(&self) -> bool {
//             self.can_focus
//         }
//         fn can_focus(&self) -> bool {
//             self.can_focus
//         }
//     }

//     impl ViewComposer for MockComponent {}
// }
