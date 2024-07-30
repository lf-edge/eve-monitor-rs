use std::cell::RefCell;

use crate::{ipc::message::IpcMessage, raw_model::RawModel};

pub type Model = RefCell<MonitorModel>;
#[derive(Debug)]
pub struct MonitorModel {
    pub dmesg: Vec<rmesg::entry::Entry>,
}

impl MonitorModel {
    pub fn update_from_raw_model(&mut self, _raw_model: &RawModel) {}
}

impl Default for MonitorModel {
    fn default() -> Self {
        MonitorModel {
            dmesg: Vec::with_capacity(1000),
        }
    }
}

