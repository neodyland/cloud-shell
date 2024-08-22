use std::{env};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get},
    Router,
};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use kube::{core::subresource::AttachParams, Client};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tracing_subscriber::{fmt::time::LocalTime, EnvFilter};

use manager::k8s::{delete_all_pods, Shell};

mod manager;
mod types;

fn json_to_msg<T>(data: &T) -> anyhow::Result<Message>
where
    T: ?Sized + serde::Serialize,
{
    let msg = serde_json::to_string(&data)?;
    Ok(Message::Text(msg))
}

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Now booting...");
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .init();
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    let client = Arc::new(Client::try_default().await?);

    delete_all_pods(&client, "shell".to_string()).await?;

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(ws_handle))
        .with_state(AppState {
            client: Arc::clone(&client),
        });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handle(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| wrapper(socket, Arc::clone(&state.client)))
}

async fn wrapper(ws: WebSocket, client: Arc<Client>) {
    if let Err(e) = handle_socket(ws, client).await {
        tracing::error!("{:?}", e);
    }
}

async fn handle_socket(ws: WebSocket, client: Arc<Client>) -> anyhow::Result<()> {
    let (mut write, mut read) = ws.split();
    write
        .send(json_to_msg(&types::HelloMessage {
            op: 1,
            data: "Hello, World!".to_string(),
        })?)
        .await?;
    let mut shell = Shell::builder(client)
        .namespace("shell".to_string())
        .memory_limit(env::var("MEMORY_LIMIT").unwrap_or("0.5Gi".to_string()))
        .create()
        .await?;
    shell.wait_provisioning().await?;
    write
        .send(json_to_msg(&types::ReadyMessage { op: 2, data: None })?)
        .await?;

    tracing::debug!("Pod is running, attaching to it");
    let mut attached = shell
        .pods
        .exec(
            &shell.get_pod_name(),
            vec!["/bin/bash"],
            &AttachParams {
                tty: true,
                stdin: true,
                stderr: false,
                ..Default::default()
            },
        )
        .await?;
    let mut stdout_stream = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let mut stdin_writer = attached.stdin().unwrap();
    loop {
        tokio::select! {
            msg = read.next() => {
                tracing::debug!("Received message: {:?}", msg);
                let msg = match msg {
                    Some(Ok(Message::Text(msg))) => msg,
                    Some(Ok(Message::Close(_))) => break,
                    None => break,
                    _ => continue,
                };
                let msg: serde_json::Value = serde_json::from_str(&msg)?;
                let op = msg["op"].as_u64().unwrap();
                match op {
                    3 => {
                        tracing::debug!("Running command: {:?}", msg["data"]);
                        let data = msg["data"].as_str().unwrap_or("");
                        stdin_writer.write_all(data.as_bytes()).await?;
                    }
                    _ => {}
                }
            }
            stdout = stdout_stream.try_next() => {
                let stdout = match stdout {
                    Ok(Some(stdout)) => stdout,
                    _ => break,
                };
                write.send(json_to_msg(&types::RunCommandMessage {
                    op: 4,
                    data: String::from_utf8_lossy(&stdout).to_string(),
                })?).await?;
            }
        }
    }
    shell.close().await?;

    Ok(())
}
