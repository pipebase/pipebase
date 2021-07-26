use crate::model::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::{Event, Pod};
use kube::{
    api::{Api, ListParams, LogParams},
    Client,
};
use kube_runtime::{utils::try_flatten_applied, watcher};
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
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
    tx: Option<Sender<KubeLog>>,
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
impl Listen<KubeLog, KubeLogReaderConfig> for KubeLogReader {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_log().await
    }

    fn set_sender(&mut self, sender: Sender<KubeLog>) {
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
                    let log = KubeLog::new(
                        self.namespace.to_owned(),
                        self.pod.to_owned(),
                        self.container.to_owned(),
                        String::from_utf8(line.to_vec())?,
                    );
                    tx.send(log).await?;
                }
                None => (),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct KubeEventReaderConfig {
    // leave as empty if monitor all namespaces
    namespace: String,
}

#[async_trait]
impl FromPath for KubeEventReaderConfig {}

impl ConfigInto<KubeEventReader> for KubeEventReaderConfig {}

pub struct KubeEventReader {
    events: Api<Event>,
    tx: Option<Sender<KubeEvent>>,
}

#[async_trait]
impl FromConfig<KubeEventReaderConfig> for KubeEventReader {
    async fn from_config(config: KubeEventReaderConfig) -> anyhow::Result<Self> {
        let client = Client::try_default().await?;
        let namespace = config.namespace;
        let events: Api<Event> = match namespace.is_empty() {
            false => Api::namespaced(client, &namespace),
            true => Api::all(client),
        };
        Ok(KubeEventReader { events, tx: None })
    }
}

#[async_trait]
impl Listen<KubeEvent, KubeEventReaderConfig> for KubeEventReader {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await
    }

    fn set_sender(&mut self, sender: Sender<KubeEvent>) {
        self.tx = Some(sender)
    }
}

impl KubeEventReader {
    async fn do_run(&self) -> anyhow::Result<()> {
        let params = ListParams::default();
        let mut watcher = try_flatten_applied(watcher(self.events.to_owned(), params)).boxed();
        let tx = self
            .tx
            .as_ref()
            .expect("sender not inited for kube event reader");
        while let Some(event) = watcher.try_next().await? {
            let namespace = event.involved_object.namespace.unwrap_or_default();
            let kind = event.involved_object.kind.unwrap_or_default();
            let name = event.involved_object.name.unwrap_or_default();
            let message = event.message.unwrap_or_default();
            let action = event.action.unwrap_or_default();
            let count = event.count.unwrap_or_default();
            let component = event.reporting_component.unwrap_or_default();
            let instance = event.reporting_instance.unwrap_or_default();
            let event_time: DateTime<Utc> = match event.event_time {
                Some(event_time) => event_time.0,
                None => Utc::now(),
            };
            tx.send(KubeEvent::new(
                namespace, kind, name, message, action, count, component, instance, event_time,
            ))
            .await?;
        }
        Ok(())
    }
}
