use super::Select;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Deserialize)]
pub struct DefaultHashSelectorConfig {}

#[async_trait]
impl FromPath for DefaultHashSelectorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(DefaultHashSelectorConfig {})
    }
}

#[async_trait]
impl ConfigInto<DefaultHashSelector> for DefaultHashSelectorConfig {}

/// Select candidate by it's hash value
pub struct DefaultHashSelector {}

#[async_trait]
impl FromConfig<DefaultHashSelectorConfig> for DefaultHashSelector {
    async fn from_config(_config: DefaultHashSelectorConfig) -> anyhow::Result<Self> {
        Ok(DefaultHashSelector {})
    }
}

/// # Parameters
/// * T: input
#[async_trait]
impl<T> Select<T, DefaultHashSelectorConfig> for DefaultHashSelector
where
    T: Hash + Sync,
{
    /// `candidates`: index of downstreams
    /// `t`: input data reference
    async fn select(&mut self, t: &T, candidates: &[&usize]) -> anyhow::Result<Vec<usize>> {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        let h = hasher.finish();
        let i = h % (candidates.len() as u64);
        Ok(vec![candidates[i as usize].to_owned()])
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;
    use tokio::sync::mpsc::{Receiver, Sender};

    #[derive(Clone, Debug, HashedBy)]
    struct Record {
        #[hash]
        pub key: String,
        pub value: i32,
    }

    async fn populate_records(tx: Sender<Record>, records: Vec<Record>) {
        for record in records {
            tx.send(record).await.unwrap();
        }
    }

    async fn receive_records(mut rx: Receiver<Record>, id: usize) -> usize {
        let mut c: usize = 0;
        loop {
            match rx.recv().await {
                Some(record) => {
                    c += 1;
                    println!("id: {}, record {:#?}", id, record);
                }
                None => return c,
            }
        }
    }

    #[tokio::test]
    async fn test_hash_select() {
        let (tx0, rx0) = channel!(Record, 1024);
        let (tx1, rx1) = channel!(Record, 1024);
        let (tx2, rx2) = channel!(Record, 1024);
        let channels = pipe_channels!(rx0, [tx1, tx2]);
        // 123 -> id1, abc -> id2 if hashkey is "key" only
        // abc, 1 -> id1, others -> id2 if hashkey is (key, value) combined
        let records = vec![
            Record {
                key: "abc".to_owned(),
                value: 1,
            },
            Record {
                key: "abc".to_owned(),
                value: 2,
            },
            Record {
                key: "123".to_owned(),
                value: 1,
            },
            Record {
                key: "123".to_owned(),
                value: 2,
            },
        ];
        let f0 = populate_records(tx0, records);
        let f1 = receive_records(rx1, 1);
        let f2 = receive_records(rx2, 2);
        let pipe = selector!("hash_select");
        f0.await;
        join_pipes!([run_pipe!(pipe, DefaultHashSelectorConfig, channels)]);
        let c1 = f1.await;
        let c2 = f2.await;
        assert_eq!(4, c1 + c2)
    }
}
