use chrono::{DateTime, Utc};
use pipebase::common::{Convert, Pair};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KubeLog {
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub log: String,
}

impl KubeLog {
    pub fn new(namespace: String, pod: String, container: String, log: String) -> Self {
        KubeLog {
            namespace,
            pod,
            container,
            log,
        }
    }
}

impl Convert<KubeLog> for Pair<String, String> {
    fn convert(rhs: KubeLog) -> Self {
        let l = format!("{}/{}/{}", rhs.namespace, rhs.pod, rhs.container);
        let r = rhs.log;
        Pair::<String, String>::new(l, r)
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

impl Convert<KubeEvent> for Pair<String, KubeEvent> {
    fn convert(rhs: KubeEvent) -> Self {
        let l = format!("{}/{}/{}", rhs.namespace, rhs.kind, rhs.name);
        Pair::<String, KubeEvent>::new(l, rhs)
    }
}

impl KubeEvent {
    pub fn new(
        namespace: String,
        kind: String,
        name: String,
        message: String,
        action: String,
        count: i32,
        component: String,
        instance: String,
        event_time: DateTime<Utc>,
    ) -> Self {
        KubeEvent {
            namespace,
            kind,
            name,
            message,
            action,
            count,
            component,
            instance,
            event_time,
        }
    }
}
