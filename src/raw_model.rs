use std::cell::{Ref, RefCell};

use crate::ipc::eve_types::{DeviceNetworkStatus, DevicePortConfigList, NetworkPortConfig};

struct MonitorModel {
    port_config: Option<NetworkPortConfig>,
    dpc_list: Option<DevicePortConfigList>,
    network_status: Option<DeviceNetworkStatus>,
}

pub struct RawModel(RefCell<MonitorModel>);

impl RawModel {
    pub fn new() -> Self {
        RawModel(RefCell::new(MonitorModel {
            port_config: None,
            dpc_list: None,
            network_status: None,
        }))
    }

    pub fn set_dpc_list(&self, dpc_list: DevicePortConfigList) {
        self.0.borrow_mut().dpc_list = Some(dpc_list);
    }

    pub fn set_network_status(&self, network_status: DeviceNetworkStatus) {
        self.0.borrow_mut().network_status = Some(network_status);
    }

    pub fn set_port_config(&self, port_config: NetworkPortConfig) {
        self.0.borrow_mut().port_config = Some(port_config);
    }

    pub fn get_dpc_list(&self) -> Option<Ref<DevicePortConfigList>> {
        let borrow = self.0.borrow();
        if borrow.dpc_list.is_some() {
            Some(Ref::map(borrow, |monitor_model| {
                monitor_model.dpc_list.as_ref().unwrap()
            }))
        } else {
            None
        }
    }

    pub fn get_network_status(&self) -> Option<Ref<DeviceNetworkStatus>> {
        let borrow = self.0.borrow();
        if borrow.network_status.is_some() {
            Some(Ref::map(borrow, |monitor_model| {
                monitor_model.network_status.as_ref().unwrap()
            }))
        } else {
            None
        }
    }
}

// struct Model<'a, M: 'a>(&'a RefCell<M>);
