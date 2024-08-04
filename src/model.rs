use std::cell::RefCell;

use crate::{device::network::NetworkInterface, ipc::message::IpcMessage, raw_model::RawModel};

pub type Model = RefCell<MonitorModel>;
#[derive(Debug)]
pub struct MonitorModel {
    pub dmesg: Vec<rmesg::entry::Entry>,
    pub network: Vec<NetworkInterface>,
}

impl MonitorModel {
    fn get_network_settings(&self, raw_model: &RawModel) -> Option<Vec<NetworkInterface>> {
        let network_status = raw_model.get_network_status()?;
        let ports = network_status.ports.as_ref()?;
        Some(ports.iter().map(|p| p.into()).collect())
    }
    pub fn update_from_raw_model(&mut self, raw_model: &RawModel) {
        // we store only information enough to render the UI and
        // the ID we can use to index the raw model e.g. networking port name
        // FIXME: we can have a race condition when we get an updated raw model
        // while we display a dialog with the old one
        // we need to implement a way to handle this e.g. check update time or
        // better do "almost equal" comparison algorithm
        self.network = self.get_network_settings(raw_model).unwrap_or_default();
    }
}

impl Default for MonitorModel {
    fn default() -> Self {
        MonitorModel {
            dmesg: Vec::with_capacity(1000),
            network: Vec::new(),
        }
    }
}
