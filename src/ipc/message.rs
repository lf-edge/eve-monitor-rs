use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

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

#[derive(Debug, Deserialize)]
struct Data {
    ip: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    Connecting,
    Ready,
    Request(Request),
    Response(Response),
    AsyncNotify(Response),
}

impl IpcMessage {
    fn from_reader(bytes: Bytes) -> Self {
        match serde_json::from_reader(bytes.reader()) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                Self::Response(Response {
                    result: None,
                    error: Some("Failed to parse message".to_string()),
                })
            }
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
