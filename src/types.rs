use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HelloMessage {
    pub op: u8,
    pub data: String,
}

#[derive(Serialize)]
pub struct IdentifyMessage {
    pub op: u8,
    pub data: Option<u8>,
}

#[derive(Serialize)]
pub struct ReadyMessage {
    pub op: u8,
    pub data: Option<u8>,
}

#[derive(Serialize)]
pub struct RunCommandMessage {
    pub op: u8,
    pub data: String,
}
