use futures_util::{StreamExt, TryStreamExt};
use k8s_openapi::{
    api::core::v1::{Container, Pod, PodSpec, ResourceRequirements},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};
use kube::{
    api::{DeleteParams, ListParams, WatchParams},
    core::{params::PostParams, WatchEvent},
    Api, Client, ResourceExt,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub async fn delete_all_pods(client: &Client, namespace: String) -> anyhow::Result<()> {
    let pods: Api<Pod> = Api::namespaced(client.to_owned(), &namespace);
    let all_pods = pods.list(&ListParams::default()).await?;
    for pod in all_pods {
        pods.delete(&pod.name_any(), &DeleteParams::default())
            .await?;
    }
    Ok(())
}

pub struct ShellBuilder {
    memory_limit: String,
    client: Arc<Client>,
    namespace: String,
}

impl ShellBuilder {
    fn new(client: Arc<Client>) -> Self {
        Self {
            memory_limit: "0.5Gi".to_string(),
            client,
            namespace: "default".to_string(),
        }
    }

    pub fn memory_limit(mut self, memory_limit: String) -> ShellBuilder {
        self.memory_limit = memory_limit;
        self
    }

    pub fn namespace(mut self, namespace: String) -> ShellBuilder {
        self.namespace = namespace;
        self
    }

    pub async fn create(&self) -> anyhow::Result<Shell> {
        let pod_id = uuid::Uuid::new_v4();
        let pods: Api<Pod> = Api::namespaced((*self.client).clone(), &self.namespace);
        let shell = Shell {
            memory_limit: self.memory_limit.clone(),
            pod_id: pod_id.to_string(),
            pods,
            provisioned: false,
        };
        shell.create().await?;
        Ok(shell)
    }
}
pub struct Shell {
    memory_limit: String,
    pod_id: String,
    pub pods: Api<Pod>,
    pub provisioned: bool,
}

impl Shell {
    pub fn builder(client: Arc<Client>) -> ShellBuilder {
        ShellBuilder::new(client)
    }

    async fn create(&self) -> anyhow::Result<()> {
        let pod_name = self.get_pod_name();
        let mut resource_limits = BTreeMap::new();
        resource_limits.insert("memory".to_string(), Quantity(self.memory_limit.clone()));
        self.pods
            .create(
                &PostParams::default(),
                &Pod {
                    metadata: ObjectMeta {
                        name: Some(pod_name.clone()),
                        ..Default::default()
                    },
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "shell".to_string(),
                            image: Some("ghcr.io/tuna2134/cloud-shell-arch".to_string()),
                            command: Some(vec!["sleep".to_string(), "infinity".to_string()]),
                            resources: Some(ResourceRequirements {
                                limits: Some(resource_limits),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    pub fn get_pod_name(&self) -> String {
        format!("shell-pod-{}", self.pod_id).to_string()
    }

    pub async fn wait_provisioning(&mut self) -> anyhow::Result<()> {
        let wp = WatchParams::default()
            .fields(&format!("metadata.name={}", self.get_pod_name()))
            .timeout(10);
        let mut watch = self.pods.watch(&wp, "0").await?.boxed();
        while let Some(status) = watch.try_next().await? {
            match status {
                WatchEvent::Added(pod) => {
                    tracing::info!("Added: {}", pod.name_any());
                }
                WatchEvent::Modified(pod) => {
                    tracing::info!("Modified: {:?}", pod.name_any());
                    let status = pod.status.as_ref().unwrap();
                    if status.phase.as_deref() == Some("Running") {
                        tracing::info!("Pod is running");
                        break;
                    }
                }
                _ => {}
            }
        }
        self.provisioned = true;
        Ok(())
    }

    pub async fn close(&self) -> anyhow::Result<()> {
        self.pods
            .delete(&self.get_pod_name(), &Default::default())
            .await?;
        Ok(())
    }
}
