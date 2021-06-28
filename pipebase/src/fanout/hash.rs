use crate::{ConfigInto, FromConfig, FromPath, Select};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Deserialize)]
pub struct DefaultHashSelectConfig {}

impl FromPath for DefaultHashSelectConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(DefaultHashSelectConfig {})
    }
}

#[async_trait]
impl ConfigInto<DefaultHashSelect> for DefaultHashSelectConfig {}

pub struct DefaultHashSelect {}

#[async_trait]
impl FromConfig<DefaultHashSelectConfig> for DefaultHashSelect {
    async fn from_config(_config: &DefaultHashSelectConfig) -> anyhow::Result<Self> {
        Ok(DefaultHashSelect {})
    }
}

impl<T: Hash> Select<T, DefaultHashSelectConfig> for DefaultHashSelect {
    fn select<'a>(&mut self, t: &T, candidates: &'a [&'a usize]) -> &'a [&'a usize] {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        let h = hasher.finish();
        let i = h % (candidates.len() as u64);
        let i = i as usize;
        &candidates[i..i + 1]
    }
}

#[cfg(test)]
mod tests {

    use super::DefaultHashSelectConfig;
    use crate::HashKey;
    use crate::{channel, selector, spawn_join, FromPath, Pipe, Selector};
    use std::hash::{Hash, Hasher};
    use tokio::sync::mpsc::{Receiver, Sender};

    #[derive(Clone, Debug, HashKey)]
    struct Record {
        #[hkey]
        pub key: String,
        pub value: i32,
    }

    async fn populate_records(tx: &mut Sender<Record>, records: Vec<Record>) {
        for record in records {
            tx.send(record).await.unwrap();
        }
    }

    async fn receive_records(rx: &mut Receiver<Record>, id: usize) -> usize {
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
        let (mut tx0, rx0) = channel!(Record, 1024);
        let (tx1, mut rx1) = channel!(Record, 1024);
        let (tx2, mut rx2) = channel!(Record, 1024);
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
        let f0 = populate_records(&mut tx0, records);
        let f1 = receive_records(&mut rx1, 1);
        let f2 = receive_records(&mut rx2, 2);
        let mut pipe = selector!("hash_select", DefaultHashSelectConfig, rx0, [tx1, tx2]);
        f0.await;
        drop(tx0);
        spawn_join!(pipe);
        let c1 = f1.await;
        let c2 = f2.await;
        assert_eq!(4, c1 + c2)
    }
}
