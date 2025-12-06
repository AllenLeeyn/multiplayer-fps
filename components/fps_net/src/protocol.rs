use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Ping = 0,
    Pong = 1,

    ConnectRequest = 2,
    ConnectAccept = 3,
    ConnectDeny = 4,
    DisconnectNotice = 5,

    Reliable = 6,
    Unreliable = 7,
    Acknowledgement = 8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct MessageHeader {
    pub msg_type: MessageType,
    pub sequence: u32,
    pub timestamp_ms: u64,
}
