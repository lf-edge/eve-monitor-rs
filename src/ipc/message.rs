// TODO: uncomment to use with serde_json::from_reader
// use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use log::error;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use super::eve_types::DeviceNetworkStatus;
use super::eve_types::DevicePortConfigList;

#[derive(Debug, Serialize, Deserialize)]
pub enum RpcCommand {
    Ping,
    GetData,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub command: RpcCommand,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    result: Option<Value>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum IpcMessage {
    Connecting,
    Ready,
    Request(Request),
    Response(Response),
    NetworkStatus(DeviceNetworkStatus),
    DPCList(DevicePortConfigList),
}

impl IpcMessage {
    fn from_reader(bytes: Bytes) -> Self {
        // TODO: it is faster to call serde_json::from_reader directly
        // but I want to log the message if it fails to parse
        if let Ok(s) = String::from_utf8(bytes.to_vec()) {
            match serde_json::from_str(s.as_str()) {
                Ok(message) => message,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    error!("MESSAGE: {}", s);
                    Self::Response(Response {
                        result: None,
                        error: Some("Failed to parse message".to_string()),
                    })
                }
            }
        } else {
            Self::Response(Response {
                result: None,
                error: Some("Failed to parse message to utf8".to_string()),
            })
        }
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