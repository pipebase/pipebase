use crate::{utils::IntoBytes, RocksDBGroupTable};
use async_trait::async_trait;
use pipebase::{
    AggregateAs, Averagef32, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Map, Pair,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::{hash::Hash, iter::FromIterator};

#[derive(Deserialize)]
pub struct RocksDBUnorderedGroupAveragef32AggregatorConfig {
    path: String,
}

impl FromPath for RocksDBUnorderedGroupAveragef32AggregatorConfig {}

impl ConfigInto<RocksDBUnorderedGroupAveragef32Aggregator>
    for RocksDBUnorderedGroupAveragef32AggregatorConfig
{
}

pub struct RocksDBUnorderedGroupAveragef32Aggregator {
    path: String,
}

#[async_trait]
impl FromConfig<RocksDBUnorderedGroupAveragef32AggregatorConfig>
    for RocksDBUnorderedGroupAveragef32Aggregator
{
    async fn from_config(
        config: &RocksDBUnorderedGroupAveragef32AggregatorConfig,
    ) -> anyhow::Result<Self> {
        Ok(RocksDBUnorderedGroupAveragef32Aggregator {
            path: config.path.to_owned(),
        })
    }
}

impl<I, T, K, U> GroupAggregate<I, T, K, Averagef32, U, RocksDBGroupTable<HashMap<K, Averagef32>>>
    for RocksDBUnorderedGroupAveragef32Aggregator
where
    I: GroupAs<K> + AggregateAs<Averagef32>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq + IntoBytes + Clone,
    U: FromIterator<Pair<K, Averagef32>>,
{
    fn merge(&self, v: &mut Averagef32, i: &I) {
        *v += i.aggregate_value()
    }

    fn group_table(&self) -> anyhow::Result<RocksDBGroupTable<HashMap<K, Averagef32>>> {
        RocksDBGroupTable::new(&self.path, HashMap::new())
    }
}

#[async_trait]
impl<I, T, K> Map<T, Vec<Pair<K, Averagef32>>, RocksDBUnorderedGroupAveragef32AggregatorConfig>
    for RocksDBUnorderedGroupAveragef32Aggregator
where
    I: GroupAs<K> + AggregateAs<Averagef32>,
    T: IntoIterator<Item = I> + Send + 'static,
    K: Hash + Eq + PartialEq + IntoBytes + Clone,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, Averagef32>>> {
        Ok(self.group_aggregate(data)?)
    }
}

#[cfg(test)]
mod unordered_group_avg_f32_tests {

    use crate::*;
    use pipebase::*;
    use tokio::sync::mpsc::Sender;

    #[derive(Clone, Debug, AggregateAs, GroupAs)]
    struct Record {
        #[group]
        id: String,
        #[agg(avgf32)]
        value: i32,
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
    async fn test_unordered_group_avg_f32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Averagef32>>, 1024);
        let mut pipe = mapper!("group_avg_f32");
        let pipe = run_pipe!(
            pipe,
            RocksDBUnorderedGroupAveragef32AggregatorConfig,
            "resources/catalogs/rocksdb_avg.yml",
            [tx1],
            rx0
        );
        let f0 = populate_records(
            tx0,
            vec![
                vec![
                    Record {
                        id: "foo".to_owned(),
                        value: 1,
                    },
                    Record {
                        id: "foo".to_owned(),
                        value: 2,
                    },
                    Record {
                        id: "bar".to_owned(),
                        value: 2,
                    },
                    Record {
                        id: "bar".to_owned(),
                        value: 3,
                    },
                ],
                vec![
                    Record {
                        id: "foo".to_owned(),
                        value: 3,
                    },
                    Record {
                        id: "bar".to_owned(),
                        value: 4,
                    },
                ],
            ],
        );
        f0.await;
        join_pipes!([pipe]);
        let group_avgs = rx1.recv().await.expect("group average not found");
        for avg in group_avgs {
            match &avg.left()[..] {
                "foo" => {
                    assert_eq!(1.5, avg.right().average())
                }
                "bar" => {
                    assert_eq!(2.5, avg.right().average())
                }
                _ => unreachable!("unexpected group {}", avg.left()),
            }
        }
        // stateful group average
        let group_avgs = rx1.recv().await.expect("group average not found");
        for avg in group_avgs {
            match &avg.left()[..] {
                "foo" => {
                    assert_eq!(2.0, avg.right().average())
                }
                "bar" => {
                    assert_eq!(3.0, avg.right().average())
                }
                _ => unreachable!("unexpected group {}", avg.left()),
            }
        }
        std::fs::remove_dir_all("resources/data/rocks/avg").unwrap()
    }
}
