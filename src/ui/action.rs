use crate::traits::IAction;
#[derive(Debug, Clone)]
pub enum UiActions<A> {
    Quit,
    Redraw,
    CheckBox { checked: bool },
    RadioGroup { selected: usize },
    Input { text: String },
    ButtonClicked(String),
    DismissDialog,
    UserAction(A),
}

impl<A> UiActions<A> {
    pub fn new_user_action(action: A) -> Self {
        UiActions::UserAction(action)
    }
}

#[derive(Debug, Clone)]
pub struct Action<A> {
    pub source: String,
    pub target: Option<String>,
    pub action: UiActions<A>,
}

impl<A> Action<A> {
    pub fn new_user_action<S: Into<String>>(source: S, action: A) -> Self {
        Self {
            source: source.into(),
            action: UiActions::UserAction(action),
            target: None,
        }
    }
    pub fn new<S: Into<String>>(source: S, action: UiActions<A>) -> Self {
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

impl<A> IAction for Action<A>
where
    A: Clone,
{
    type Target = UiActions<A>;
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
