use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

mod types;

fn json_to_msg<T>(data: &T) -> anyhow::Result<Message>
where
    T: ?Sized + serde::Serialize,
{
    let msg = serde_json::to_string(&data)?;
    Ok(Message::Text(msg))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        ws_handler(stream).await?;
    }
    Ok(())
}

async fn ws_handler(stream: TcpStream) -> anyhow::Result<()> {
    let ws = accept_async(stream).await?;
    let (mut write, _read) = ws.split();
    write
        .send(json_to_msg(&types::HelloMessage {
            op: 1,
            data: "Hello, World!".to_string(),
        })?)
        .await?;
    Ok(())
}
