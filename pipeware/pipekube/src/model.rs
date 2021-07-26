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
    pub kind: String,
    pub name: String,
    pub message: String,
}

impl Convert<KubeEvent> for Pair<String, String> {
    fn convert(rhs: KubeEvent) -> Self {
        let l = format!("{}/{}", rhs.kind, rhs.name);
        let r = rhs.message;
        Pair::<String, String>::new(l, r)
    }
}
