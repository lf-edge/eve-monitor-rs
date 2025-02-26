// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

// TODO: uncomment to use with serde_json::from_reader
// use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use log::error;
use serde::Deserialize;
use serde::Serialize;

use super::eve_types::AppInstanceStatus;
use super::eve_types::AppInstanceSummary;
use super::eve_types::AppsList;
use super::eve_types::DeviceNetworkStatus;
use super::eve_types::DevicePortConfig;
use super::eve_types::DevicePortConfigList;
use super::eve_types::DownloaderStatus;
use super::eve_types::EveNodeStatus;
use super::eve_types::EveOnboardingStatus;
use super::eve_types::EveVaultStatus;
use super::eve_types::LedBlinkCounter;
use super::eve_types::PhysicalIOAdapterList;
use super::eve_types::ZedAgentStatus;

/// WindowId is a unique identifier for a window that is incremented sequentially.
pub type RequestId = u64;

struct RequestIdGenerator(AtomicU64);
impl RequestIdGenerator {
    fn next(&self) -> RequestId {
        self.0.fetch_add(1, Ordering::SeqCst)
    }
}

// statically initialize the window id counter
static REQ_ID: RequestIdGenerator = RequestIdGenerator(AtomicU64::new(1));

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "RequestType", content = "RequestData")]
pub enum Request {
    SetDPC(DevicePortConfig),
    SetServer(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum IpcMessage {
    Connecting,
    Ready,
    NetworkStatus(DeviceNetworkStatus),
    DPCList(DevicePortConfigList),
    DownloaderStatus(DownloaderStatus),
    IOAdapters(PhysicalIOAdapterList),
    AppStatus(AppInstanceStatus),
    AppSummary(AppInstanceSummary),
    VaultStatus(EveVaultStatus),
    OnboardingStatus(EveOnboardingStatus),
    LedBlinkCounter(LedBlinkCounter),
    NodeStatus(EveNodeStatus),
    AppsList(AppsList),
    ZedAgentStatus(ZedAgentStatus),
    Response {
        #[serde(flatten)]
        result: core::result::Result<String, String>,
        id: u64,
    },
    #[serde(untagged)]
    Request {
        #[serde(flatten)]
        request: Request,
        id: u64,
    },
}

// static mutable  variable to store the index of log file to write
//TODO: it will go away eventually
static mut LOG_FILE_INDEX: u64 = 0;

fn dump_to_file(message: &str, is_error: bool) {
    use std::fs::OpenOptions;
    use std::io::Write;

    // get EVE_MONITOR_LOG_DIR from environment
    if let Ok(log_dir) = std::env::var("EVE_MONITOR_LOG_DIR") {
        let log_file_name = format!(
            "eve_ipc_message{}-{}.json",
            if is_error { "-err" } else { "" },
            unsafe { LOG_FILE_INDEX }
        );
        let log_file_name = std::path::Path::new(log_dir.as_str()).join(log_file_name);
        // increment log file index
        unsafe {
            LOG_FILE_INDEX += 1;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_name)
            .unwrap();
        file.write_all(message.as_bytes()).unwrap();
        return;
    }
}

impl IpcMessage {
    fn from_reader(bytes: Bytes) -> Self {
        // TODO: it is faster to call serde_json::from_reader directly
        // but I want to log the message if it fails to parse
        if let Ok(s) = String::from_utf8(bytes.to_vec()) {
            match serde_json::from_str(s.as_str()) {
                Ok(message) => {
                    dump_to_file(s.as_str(), false);
                    message
                }
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    error!("MESSAGE: {}", s);
                    dump_to_file(s.as_str(), true);
                    Self::Response {
                        id: 0,
                        result: Err("Failed to parse message".to_string()),
                    }
                }
            }
        } else {
            Self::Response {
                id: 0,
                result: Err("Failed to parse message to utf8".to_string()),
            }
        }
    }

    pub fn new_request(request: Request) -> Self {
        let id = REQ_ID.next();
        Self::Request { request, id }
    }
}

impl From<Bytes> for IpcMessage {
    fn from(bytes: Bytes) -> Self {
        Self::from_reader(bytes)
    }
}

impl From<IpcMessage> for Bytes {
    fn from(message: IpcMessage) -> Self {
        let message = serde_json::to_string(&message).unwrap();
        Bytes::from(message)
    }
}

impl From<BytesMut> for IpcMessage {
    fn from(bytes: BytesMut) -> Self {
        Self::from_reader(bytes.freeze())
    }
}
