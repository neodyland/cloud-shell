use std::collections::BTreeMap;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use k8s_openapi::{
    api::core::v1::{Container, Pod, PodSpec, ResourceRequirements},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
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
    let pod_name = format!("shell-pod-{}", uuid::Uuid::new_v4()).to_string();
    let mut resource_limits = BTreeMap::new();
    resource_limits.insert("cpu".to_string(), Quantity("0.5Gi".to_string()));
    pods.create(
        &PostParams::default(),
        &Pod {
            metadata: ObjectMeta {
                name: Some(pod_name.clone()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "shell".to_string(),
                    image: Some("archlinux".to_string()),
                    command: Some(vec!["sleep".to_string(), "infinity".to_string()]),
                    resources: Some(
                        ResourceRequirements {
                            limits: Some(resource_limits),
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .await?;
    let wp = WatchParams::default()
        .fields(&format!("metadata.name={}", pod_name))
        .timeout(10);
    let mut watch = pods.watch(&wp, "0").await?.boxed();
    while let Some(status) = watch.try_next().await? {
        match status {
            WatchEvent::Added(pod) => {
                println!("Added: {}", pod.name_any());
            }
            WatchEvent::Modified(pod) => {
                println!("Modified: {:?}", pod.name_any());
                let status = pod.status.as_ref().unwrap();
                if status.phase.as_deref() == Some("Running") {
                    println!("Pod is running");
                    break;
                }
            }
            _ => {}
        }
    }
    write
        .send(json_to_msg(&types::ReadyMessage { op: 2, data: None })?)
        .await?;

    println!("Pod is running, attaching to it");
    let mut attached = pods
        .exec(
            &pod_name,
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
                println!("Received message: {:?}", msg);
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
                        println!("Running command: {:?}", msg["data"]);
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
    pods.delete(&pod_name, &Default::default()).await?;

    Ok(())
}
