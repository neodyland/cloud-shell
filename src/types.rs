use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
pub enum ServerMessage {
    Hello(String),
    Identify(Option<u8>),
    Ready(Option<u8>),
    Stdout(Vec<u8>),
}
#[derive(Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum ClientMessage {
    Stdin(Vec<u8>),
}
