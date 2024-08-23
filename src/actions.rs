use crate::ui::ipdialog::IpDialogState;

#[derive(Debug, Clone, PartialEq)]
pub struct MainWndState {
    pub a: u32,
    pub ip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MonActions {
    MainWndStateUpdated(MainWndState),
    NetworkInterfaceUpdated(IpDialogState),
}
