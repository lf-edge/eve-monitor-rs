use uuid::Uuid;

use crate::ipc::eve_types::{DeviceNetworkStatus, DevicePortConfig, DevicePortConfigList};
#[derive(Debug, Default)]
pub struct RawModel {
    dpc_list: Option<DevicePortConfigList>,
    network_status: Option<DeviceNetworkStatus>,
}

impl RawModel {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_dpc_list(&mut self, dpc_list: DevicePortConfigList) {
        self.dpc_list = Some(dpc_list);
    }

    pub fn get_dpc_list(&self) -> Option<&DevicePortConfigList> {
        self.dpc_list.as_ref()
    }

    fn get_network_status(&self) -> Option<&DeviceNetworkStatus> {
        self.network_status.as_ref()
    }

    pub fn get_current_dpc(&self) -> Option<&DevicePortConfig> {
        let net_status = self.get_network_status()?;
        let key = &net_status.dpc_key;
        self.get_dpc_list()?.get_dpc_by_key(key)
    }
}
