use futures_util::{SinkExt, StreamExt, TryStreamExt};
use k8s_openapi::{
    api::core::v1::{Container, Pod, PodSpec},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{
    api::WatchParams,
    core::{params::PostParams, subresource::AttachParams, WatchEvent},
    Api, Client, ResourceExt,
};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
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
    write
        .send(json_to_msg(&types::ReadyMessage { op: 2, data: None })?)
        .await?;

    let wp = WatchParams::default()
        .fields("metadata.name=shell-pod")
        .timeout(10);
    let mut watch = pods.watch(&wp, "0").await?.boxed();
    while let Some(status) = watch.try_next().await? {
        match status {
            WatchEvent::Added(pod) => {
                println!("Added: {}", pod.name_any());
            }
            WatchEvent::Modified(pod) => {
                println!("Modified: {}", pod.name_any());
                let status = pod.status.as_ref().unwrap();
                if status.phase.as_deref() == Some("Running") {
                    break;
                }
                write
                    .send(json_to_msg(&types::ReadyMessage { op: 2, data: None })?)
                    .await?;
            }
            _ => {}
        }
    }

    let mut attached = pods
        .exec(
            "/bin/bash",
            vec!["/bin/bash"],
            &AttachParams {
                tty: true,
                stdin: true,
                ..Default::default()
            },
        )
        .await?;

    loop {
        tokio::select! {
            msg = read.next() => {
                let msg = if let Some(Ok(Message::Text(msg))) = msg {
                    msg
                } else {
                    continue;
                };
                let msg: serde_json::Value = serde_json::from_str(&msg)?;
                let op = msg["op"].as_u64().unwrap();
                match op {
                    3 => {
                        let data = msg["data"].as_str().unwrap();
                        attached.stdin().unwrap().write_all(data.as_bytes()).await?;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
