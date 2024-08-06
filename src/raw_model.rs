use crate::ipc::eve_types::{
    DeviceNetworkStatus, DevicePortConfig, DevicePortConfigList, PhysicalIOAdapterList,
};
#[derive(Debug)]
pub struct RawModel {
    dpc_list: Option<DevicePortConfigList>,
    network_status: Option<DeviceNetworkStatus>,
    io_adapters: Option<PhysicalIOAdapterList>,
}

impl RawModel {
    pub fn new() -> Self {
        Self {
            dpc_list: None,
            network_status: None,
            io_adapters: None,
        }
    }

    pub fn set_dpc_list(&mut self, dpc_list: DevicePortConfigList) {
        self.dpc_list = Some(dpc_list);
    }

    pub fn set_network_status(&mut self, network_status: DeviceNetworkStatus) {
        self.network_status = Some(network_status);
    }

    pub fn set_io_adapters(&mut self, io_adapters: PhysicalIOAdapterList) {
        self.io_adapters = Some(io_adapters);
    }

    pub fn get_dpc_list(&self) -> Option<&DevicePortConfigList> {
        self.dpc_list.as_ref()
    }

    pub fn get_network_status(&self) -> Option<&DeviceNetworkStatus> {
        self.network_status.as_ref()
    }

    pub fn get_io_adapters(&self) -> Option<&PhysicalIOAdapterList> {
        self.io_adapters.as_ref()
    }

    pub fn get_current_dpc(&self) -> Option<&DevicePortConfig> {
        let net_status = self.get_network_status()?;
        let key = &net_status.dpc_key;
        self.get_dpc_list()?.get_dpc_by_key(key)
    }
}
