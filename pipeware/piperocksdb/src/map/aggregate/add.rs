use super::table::RocksDBGroupTable;
use async_trait::async_trait;
use pipebase::{
    common::{AggregateAs, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Init, Pair},
    map::Map,
};
use pipebytes::{AsBytes, FromBytes};
use serde::Deserialize;
use std::collections::HashMap;
use std::{hash::Hash, iter::FromIterator};

#[derive(Deserialize)]
pub struct RocksDBUnorderedGroupAddAggregatorConfig {
    path: String,
}

impl FromPath for RocksDBUnorderedGroupAddAggregatorConfig {}

impl ConfigInto<RocksDBUnorderedGroupAddAggregator> for RocksDBUnorderedGroupAddAggregatorConfig {}

pub struct RocksDBUnorderedGroupAddAggregator {
    path: String,
}

#[async_trait]
impl FromConfig<RocksDBUnorderedGroupAddAggregatorConfig> for RocksDBUnorderedGroupAddAggregator {
    async fn from_config(config: RocksDBUnorderedGroupAddAggregatorConfig) -> anyhow::Result<Self> {
        Ok(RocksDBUnorderedGroupAddAggregator { path: config.path })
    }
}

impl<I, T, K, V, U> GroupAggregate<I, T, K, V, U, RocksDBGroupTable<HashMap<K, V>>>
    for RocksDBUnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + AsBytes,
    V: std::ops::AddAssign<V> + Init + AsBytes + FromBytes + Clone,
    T: IntoIterator<Item = I>,
    U: FromIterator<Pair<K, V>>,
{
    fn merge(&self, v: &mut V, i: &I) {
        *v += i.aggregate_value();
    }

    fn group_table(&self) -> anyhow::Result<RocksDBGroupTable<HashMap<K, V>>> {
        RocksDBGroupTable::new(&self.path, HashMap::new())
    }
}

#[async_trait]
impl<I, K, V, T> Map<T, Vec<Pair<K, V>>, RocksDBUnorderedGroupAddAggregatorConfig>
    for RocksDBUnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + AsBytes,
    V: std::ops::AddAssign<V> + Init + FromBytes + AsBytes + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        let sums = self.group_aggregate(data)?;
        Ok(sums)
    }
}

#[cfg(test)]
mod rockdb_group_sum_tests {

    use crate::*;
    use pipebase::prelude::*;
    use tokio::sync::mpsc::Sender;

    #[derive(AggregateAs, GroupAs)]
    struct Record {
        #[group]
        id: String,
        #[agg(sum)]
        value: u32,
    }

    impl Record {
        pub fn new(id: &str, value: u32) -> Self {
            Record {
                id: id.to_owned(),
                value: value,
            }
        }
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
    async fn test_record_group_sum() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, u32>>, 1024);
        let mut pipe = mapper!("record_sum");
        let f0 = populate_records(
            tx0,
            vec![
                vec![
                    Record::new("foo", 1),
                    Record::new("foo", 2),
                    Record::new("bar", 3),
                ],
                vec![
                    Record::new("foo", 1),
                    Record::new("foo", 2),
                    Record::new("bar", 3),
                ],
            ],
        );
        f0.await;
        let pipe_run = run_pipe!(
            pipe,
            RocksDBUnorderedGroupAddAggregatorConfig,
            "resources/catalogs/rocksdb_sum.yml",
            [tx1],
            rx0
        );
        let _ = pipe_run.await;
        let gs = rx1.recv().await.unwrap();
        assert_eq!(2, gs.len());
        for sum in gs {
            match sum.left().as_str() {
                "foo" => assert_eq!(&3, sum.right()),
                "bar" => assert_eq!(&3, sum.right()),
                _ => unreachable!(),
            }
        }
        // stateful
        let gs = rx1.recv().await.unwrap();
        assert_eq!(2, gs.len());
        for sum in gs {
            match sum.left().as_str() {
                "foo" => assert_eq!(&6, sum.right()),
                "bar" => assert_eq!(&6, sum.right()),
                _ => unreachable!(),
            }
        }
        std::fs::remove_dir_all("resources/data/rocks/sum").unwrap()
    }
}

#[cfg(test)]
mod unordered_group_avg_f32_tests {

    use crate::*;
    use pipebase::prelude::*;
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
            RocksDBUnorderedGroupAddAggregatorConfig,
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

#[cfg(test)]
mod group_count32_tests {

    use crate::*;
    use pipebase::prelude::*;
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
            RocksDBUnorderedGroupAddAggregatorConfig,
            "resources/catalogs/rocksdb_count.yml",
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
        std::fs::remove_dir_all("resources/data/rocks/count").unwrap()
    }
}
