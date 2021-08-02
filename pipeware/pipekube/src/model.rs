use chrono::{DateTime, Utc};
use pipebase::common::GroupAs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KubeLog {
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub log: String,
}

impl GroupAs<String> for KubeLog {
    fn group(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.pod, self.container)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KubeEvent {
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub message: String,
    pub action: String,
    pub count: i32,
    pub component: String,
    pub instance: String,
    pub event_time: DateTime<Utc>,
}

impl GroupAs<String> for KubeEvent {
    fn group(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.kind, self.name)
    }
}

#[derive(Default)]
pub struct KubeLogBuilder {
    pub namespace: Option<String>,
    pub pod: Option<String>,
    pub container: Option<String>,
    pub log: Option<String>,
}

impl KubeLogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn pod(mut self, pod: String) -> Self {
        self.pod = Some(pod);
        self
    }

    pub fn container(mut self, container: String) -> Self {
        self.container = Some(container);
        self
    }

    pub fn log(mut self, log: String) -> Self {
        self.log = Some(log);
        self
    }

    pub fn build(self) -> KubeLog {
        KubeLog {
            namespace: self.namespace.expect("namespace not inited"),
            pod: self.pod.expect("pod not inited"),
            container: self.container.expect("container not inited"),
            log: self.log.expect("log not inited"),
        }
    }
}
