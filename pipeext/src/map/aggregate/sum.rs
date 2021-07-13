use crate::{FromBytes, IntoBytes, RocksDBGroupTable};
use async_trait::async_trait;
use pipebase::{
    AggregateAs, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Init, Map, Pair,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::{hash::Hash, iter::FromIterator};

#[derive(Deserialize)]
pub struct RocksDBUnorderedGroupSumAggregatorConfig {
    path: String,
}

impl FromPath for RocksDBUnorderedGroupSumAggregatorConfig {}

impl ConfigInto<RocksDBUnorderedGroupSumAggregator> for RocksDBUnorderedGroupSumAggregatorConfig {}

pub struct RocksDBUnorderedGroupSumAggregator {
    path: String,
}

#[async_trait]
impl FromConfig<RocksDBUnorderedGroupSumAggregatorConfig> for RocksDBUnorderedGroupSumAggregator {
    async fn from_config(
        config: &RocksDBUnorderedGroupSumAggregatorConfig,
    ) -> anyhow::Result<Self> {
        Ok(RocksDBUnorderedGroupSumAggregator {
            path: config.path.to_owned(),
        })
    }
}

impl<I, T, K, V, U> GroupAggregate<I, T, K, V, U, RocksDBGroupTable<HashMap<K, V>>>
    for RocksDBUnorderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + IntoBytes,
    V: std::ops::AddAssign<V> + Init + IntoBytes + FromBytes + Clone,
    T: IntoIterator<Item = I>,
    U: FromIterator<Pair<K, V>>,
{
    fn merge(&self, v: &mut V, i: &I) {
        *v += i.aggregate_value();
    }

    fn group_table(&self) -> anyhow::Result<RocksDBGroupTable<HashMap<K, V>>> {
        RocksDBGroupTable::new(self.path.to_owned(), HashMap::new())
    }
}

#[async_trait]
impl<I, K, V, T> Map<T, Vec<Pair<K, V>>, RocksDBUnorderedGroupSumAggregatorConfig>
    for RocksDBUnorderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + IntoBytes,
    V: std::ops::AddAssign<V> + Init + FromBytes + IntoBytes + Clone,
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
    use pipebase::*;
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
            RocksDBUnorderedGroupSumAggregatorConfig,
            "resources/catalogs/rocksdb.yml",
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
        std::fs::remove_dir_all("resources/data").unwrap()
    }
}
