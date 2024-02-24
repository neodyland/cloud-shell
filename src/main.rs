use futures_util::{SinkExt, StreamExt};
use k8s_openapi::{
    api::core::v1::{Container, Pod, PodSpec},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{core::params::PostParams, Api, Client};
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
    let client = Client::try_default().await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(ws_handler(stream, client.clone()));
    }
    Ok(())
}

async fn ws_handler(stream: TcpStream, client: Client) -> anyhow::Result<()> {
    let ws = accept_async(stream).await?;
    let (mut write, mut read) = ws.split();
    write
        .send(json_to_msg(&types::HelloMessage {
            op: 1,
            data: "Hello, World!".to_string(),
        })?)
        .await?;
    let pods: Api<Pod> = Api::namespaced(client, "shell");
    pods.create(
        &PostParams::default(),
        &Pod {
            metadata: ObjectMeta {
                name: Some("shell-pod".to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "shell".to_string(),
                    image: Some("archlinux".to_string()),
                    command: Some(vec!["sleep".to_string(), "infinity".to_string()]),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .await?;

    loop {
        tokio::select! {
            msg = read.next() => {
                println!("Recieved message");
            }
        }
    }
    Ok(())
}
