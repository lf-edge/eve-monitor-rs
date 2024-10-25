use crate::ui::ipdialog::IpDialogState;

#[derive(Debug, Clone, PartialEq)]
pub enum MonActions {
    NetworkInterfaceUpdated(IpDialogState),
}
