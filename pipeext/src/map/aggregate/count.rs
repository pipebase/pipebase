use crate::{utils::IntoBytes, RocksDBGroupTable};
use async_trait::async_trait;
use pipebase::{
    AggregateAs, ConfigInto, Count32, FromConfig, FromPath, GroupAggregate, GroupAs, Map, Pair,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::{hash::Hash, iter::FromIterator};

#[derive(Deserialize)]
pub struct RocksDBUnorderedGroupCount32AggregatorConfig {
    path: String,
}

impl FromPath for RocksDBUnorderedGroupCount32AggregatorConfig {}

impl ConfigInto<RocksDBUnorderedGroupCount32Aggregator>
    for RocksDBUnorderedGroupCount32AggregatorConfig
{
}

pub struct RocksDBUnorderedGroupCount32Aggregator {
    path: String,
}

#[async_trait]
impl FromConfig<RocksDBUnorderedGroupCount32AggregatorConfig>
    for RocksDBUnorderedGroupCount32Aggregator
{
    async fn from_config(
        config: &RocksDBUnorderedGroupCount32AggregatorConfig,
    ) -> anyhow::Result<Self> {
        Ok(RocksDBUnorderedGroupCount32Aggregator {
            path: config.path.to_owned(),
        })
    }
}

impl<I, T, K, U> GroupAggregate<I, T, K, Count32, U, RocksDBGroupTable<HashMap<K, Count32>>>
    for RocksDBUnorderedGroupCount32Aggregator
where
    I: GroupAs<K> + AggregateAs<Count32>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq + IntoBytes + Clone,
    U: FromIterator<Pair<K, Count32>>,
{
    fn merge(&self, v: &mut Count32, i: &I) {
        *v += i.aggregate_value()
    }

    fn group_table(&self) -> anyhow::Result<RocksDBGroupTable<HashMap<K, Count32>>> {
        RocksDBGroupTable::new(self.path.to_owned(), HashMap::new())
    }
}

#[async_trait]
impl<I, T, K> Map<T, Vec<Pair<K, Count32>>, RocksDBUnorderedGroupCount32AggregatorConfig>
    for RocksDBUnorderedGroupCount32Aggregator
where
    I: GroupAs<K> + AggregateAs<Count32>,
    T: IntoIterator<Item = I> + Send + 'static,
    K: Hash + Eq + PartialEq + IntoBytes + Clone,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, Count32>>> {
        Ok(self.group_aggregate(data)?)
    }
}

#[cfg(test)]
mod group_count32_tests {

    use crate::*;
    use pipebase::*;
    use tokio::sync::mpsc::Sender;

    #[derive(Debug, Clone, GroupAs, AggregateAs)]
    #[agg(count32)]
    struct Record {
        #[group]
        key: String,
    }

    async fn populate_records<T, U>(tx: Sender<T>, records: U)
    where
        U: IntoIterator<Item = T>,
    {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_group_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("group_count32");
        let pipe = run_pipe!(
            pipe,
            RocksDBUnorderedGroupCount32AggregatorConfig,
            "resources/catalogs/rocksdb.yml",
            [tx1],
            rx0
        );
        let f0 = populate_records(
            tx0,
            vec![
                vec![
                    Record {
                        key: "foo".to_owned(),
                    },
                    Record {
                        key: "foo".to_owned(),
                    },
                    Record {
                        key: "bar".to_owned(),
                    },
                ],
                vec![
                    Record {
                        key: "bar".to_owned(),
                    },
                    Record {
                        key: "bar".to_owned(),
                    },
                    Record {
                        key: "foo".to_owned(),
                    },
                ],
            ],
        );
        f0.await;
        join_pipes!([pipe]);
        let group_counts = rx1.recv().await.expect("group count32 not found");
        for count in group_counts {
            match &count.left()[..] {
                "foo" => {
                    assert_eq!(2, count.right().get())
                }
                "bar" => {
                    assert_eq!(1, count.right().get())
                }
                _ => unreachable!("unexpected group {}", count.left()),
            }
        }
        // stateful
        let group_counts = rx1.recv().await.expect("group count32 not found");
        for count in group_counts {
            match &count.left()[..] {
                "foo" => {
                    assert_eq!(3, count.right().get())
                }
                "bar" => {
                    assert_eq!(3, count.right().get())
                }
                _ => unreachable!("unexpected group {}", count.left()),
            }
        }
        std::fs::remove_dir_all("resources/data").unwrap()
    }
}
