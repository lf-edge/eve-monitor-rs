use crate::ui::ipdialog::InterfaceState;

#[derive(Debug, Clone, PartialEq)]
pub enum MonActions {
    NetworkInterfaceUpdated(InterfaceState, InterfaceState),
    ServerUpdated(String),
}
