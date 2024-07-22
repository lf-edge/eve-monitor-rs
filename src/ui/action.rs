use crate::{actions::MonActions, traits::IAction};
#[derive(Debug, Clone, PartialEq)]
pub enum UiActions {
    Quit,
    Redraw,
    CheckBox { checked: bool },
    RadioGroup { selected: usize },
    Input { text: String },
    ButtonClicked(String),
    DismissDialog,
    MonActions(MonActions),
}

#[derive(Debug, Clone)]
pub struct Action {
    pub source: String,
    pub target: Option<String>,
    pub action: UiActions,
}

impl Action {
    pub fn new<S: Into<String>>(source: S, action: UiActions) -> Self {
        Self {
            source: source.into(),
            action,
            target: None,
        }
    }
    pub fn target<S: Into<String>>(mut self, target: S) -> Self {
        self.target = Some(target.into());
        self
    }
}

impl IAction for Action {
    type Target = UiActions;
    fn get_source(&self) -> &str {
        &self.source
    }

    fn get_target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    fn split(self) -> (String, Self::Target) {
        (self.source, self.action)
    }
}
