#[derive(Debug, Clone, PartialEq)]
pub struct MainWndState {
    pub a: u32,
    pub ip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IpDialogState {
    pub ip: String,
    pub mode: String,
    pub gw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MonActions {
    MainWndStateUpdated(MainWndState),
    NetworkInterfaceUpdated(IpDialogState),
}
