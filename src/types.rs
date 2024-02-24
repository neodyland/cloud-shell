use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HelloMessage {
    pub op: u8,
    pub data: String,
}
