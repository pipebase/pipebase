use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, LogParams},
    Client,
};
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Pair},
    listen::Listen,
};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct KubeLogReaderConfig {
    namespace: String,
    pod: String,
    container: String,
}

impl FromPath for KubeLogReaderConfig {}

impl ConfigInto<KubeLogReader> for KubeLogReaderConfig {}

pub struct KubeLogReader {
    pods: Api<Pod>,
    namespace: String,
    pod: String,
    container: String,
    tx: Option<Sender<Pair<String, String>>>,
}

#[async_trait]
impl FromConfig<KubeLogReaderConfig> for KubeLogReader {
    async fn from_config(config: KubeLogReaderConfig) -> anyhow::Result<Self> {
        let client = Client::try_default().await?;
        let pods: Api<Pod> = Api::namespaced(client, &config.namespace);
        Ok(KubeLogReader {
            pods,
            namespace: config.namespace,
            pod: config.pod,
            container: config.container,
            tx: None,
        })
    }
}

#[async_trait]
impl Listen<Pair<String, String>, KubeLogReaderConfig> for KubeLogReader {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_log().await
    }

    fn set_sender(&mut self, sender: Sender<Pair<String, String>>) {
        self.tx = Some(sender)
    }
}

impl KubeLogReader {
    async fn do_log(&mut self) -> anyhow::Result<()> {
        let params = LogParams {
            container: Some(self.container.to_owned()),
            follow: true,
            tail_lines: Some(1),
            ..LogParams::default()
        };
        let mut logs = self.pods.log_stream(&self.pod, &params).await?.boxed();
        let tx = self.tx.as_ref().expect("sender not inited");
        loop {
            match logs.try_next().await? {
                Some(line) => {
                    let key = format!("{}.{}.{}", self.namespace, self.pod, self.container);
                    let value = String::from_utf8(line.to_vec())?;
                    tx.send(Pair::new(key, value)).await?;
                }
                None => (),
            }
        }
    }
}
