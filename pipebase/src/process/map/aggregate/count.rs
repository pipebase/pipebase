use crate::{
    Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Map, Pair,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::{cmp::Ordering, collections::HashMap, fmt::Debug, hash::Hash, iter::FromIterator};

#[derive(Clone, Debug, Eq)]
pub struct Count32(pub u32);

impl Count32 {
    pub fn new(c: u32) -> Self {
        Count32(c)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl std::ops::AddAssign<Self> for Count32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Ord for Count32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Count32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl PartialEq for Count32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl AggregateAs<Count32> for Count32 {
    fn aggregate_value(&self) -> Count32 {
        self.to_owned()
    }
}

impl AggregateAs<Vec<Count32>> for Count32 {
    fn aggregate_value(&self) -> Vec<Count32> {
        vec![self.to_owned()]
    }
}

impl AggregateAs<Count32> for u32 {
    fn aggregate_value(&self) -> Count32 {
        Count32::new(1)
    }
}

impl AggregateAs<Count32> for String {
    fn aggregate_value(&self) -> Count32 {
        Count32::new(1)
    }
}

#[derive(Deserialize)]
pub struct Count32AggregatorConfig {}

#[async_trait]
impl FromPath for Count32AggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(Count32AggregatorConfig {})
    }
}

impl ConfigInto<Count32Aggregator> for Count32AggregatorConfig {}

pub struct Count32Aggregator {}

#[async_trait]
impl FromConfig<Count32AggregatorConfig> for Count32Aggregator {
    async fn from_config(_config: &Count32AggregatorConfig) -> anyhow::Result<Self> {
        Ok(Count32Aggregator {})
    }
}

impl<I, T> Aggregate<I, T, Count32> for Count32Aggregator
where
    I: AggregateAs<Count32>,
    T: IntoIterator<Item = I>,
{
    fn merge(&self, u: &mut Count32, i: &I) {
        *u += i.aggregate_value()
    }
}

#[async_trait]
impl<I, T> Map<T, Count32, Count32AggregatorConfig> for Count32Aggregator
where
    I: AggregateAs<Count32>,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Count32> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod count32_tests {

    use crate::*;

    #[derive(Debug, Clone, AggregateAs)]
    #[agg(count32)]
    struct Record {}

    #[tokio::test]
    async fn test_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Count32, 1024);
        let mut pipe = mapper!("counter");
        let pipe = run_pipe!(pipe, Count32AggregatorConfig, [tx1], rx0);
        let f0 = populate_records(tx0, vec![vec![Record {}, Record {}, Record {}, Record {}]]);
        f0.await;
        join_pipes!([pipe]);
        let c = rx1.recv().await.expect("count32 not found");
        assert_eq!(4, c.get())
    }
}

#[derive(Deserialize)]
pub struct UnorderedGroupCount32AggregatorConfig {}

#[async_trait::async_trait]
impl FromPath for UnorderedGroupCount32AggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(UnorderedGroupCount32AggregatorConfig {})
    }
}

impl ConfigInto<UnorderedGroupCount32Aggregator> for UnorderedGroupCount32AggregatorConfig {}

pub struct UnorderedGroupCount32Aggregator {}

#[async_trait::async_trait]
impl FromConfig<UnorderedGroupCount32AggregatorConfig> for UnorderedGroupCount32Aggregator {
    async fn from_config(_config: &UnorderedGroupCount32AggregatorConfig) -> anyhow::Result<Self> {
        Ok(UnorderedGroupCount32Aggregator {})
    }
}

impl<I, T, K, U> GroupAggregate<I, T, K, Count32, U, HashMap<K, Count32>>
    for UnorderedGroupCount32Aggregator
where
    I: GroupAs<K> + AggregateAs<Count32>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    U: FromIterator<Pair<K, Count32>>,
{
    fn merge(&self, v: &mut Count32, i: &I) {
        *v += i.aggregate_value()
    }

    fn group_table(&self) -> anyhow::Result<HashMap<K, Count32>> {
        Ok(HashMap::new())
    }
}

#[async_trait]
impl<I, T, K> Map<T, Vec<Pair<K, Count32>>, UnorderedGroupCount32AggregatorConfig>
    for UnorderedGroupCount32Aggregator
where
    I: GroupAs<K> + AggregateAs<Count32>,
    T: IntoIterator<Item = I> + Send + 'static,
    K: Hash + Eq + PartialEq,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, Count32>>> {
        Ok(self.group_aggregate(data)?)
    }
}

#[cfg(test)]
mod group_count32_tests {

    use crate::*;

    #[derive(Debug, Clone, GroupAs, AggregateAs)]
    #[agg(count32)]
    struct Record {
        #[group]
        key: String,
    }

    #[tokio::test]
    async fn test_group_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("group_count32");
        let pipe = run_pipe!(pipe, UnorderedGroupCount32AggregatorConfig, [tx1], rx0);
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record {
                    key: "foo".to_owned(),
                },
                Record {
                    key: "foo".to_owned(),
                },
                Record {
                    key: "bar".to_owned(),
                },
            ]],
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
    }
}
