use std::hash::Hash;

use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
pub trait Aggregate<I, T, U>
where
    T: IntoIterator<Item = I>,
{
    fn aggregate(t: T) -> U;
}

#[derive(Deserialize)]
pub struct SumAggregatorConfig {}

impl FromPath for SumAggregatorConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(SumAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<SumAggregator> for SumAggregatorConfig {}

pub struct SumAggregator {}

#[async_trait]
impl FromConfig<SumAggregatorConfig> for SumAggregator {
    async fn from_config(_config: &SumAggregatorConfig) -> anyhow::Result<Self> {
        Ok(SumAggregator {})
    }
}

impl<I, T> Aggregate<I, T, I> for SumAggregator
where
    I: std::ops::AddAssign<I> + Default,
    T: IntoIterator<Item = I>,
{
    fn aggregate(t: T) -> I {
        let mut sum: I = Default::default();
        for item in t.into_iter() {
            sum += item;
        }
        sum
    }
}

#[async_trait]
impl<I, T> Map<T, I, SumAggregatorConfig> for SumAggregator
where
    I: std::ops::AddAssign<I> + Default,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<I> {
        Ok(Self::aggregate(data))
    }
}

#[cfg(test)]
mod aggregator_tests {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record(tx: Sender<Vec<usize>>, records: Vec<Vec<usize>>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<usize>, 1023);
        let (tx1, mut rx1) = channel!(usize, 1024);
        let mut pipe = mapper!("summation", SumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_record(tx0, vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]]);
        f0.await;
        spawn_join!(pipe);
        let odd = rx1.recv().await.unwrap();
        assert_eq!(16, odd);
        let even = rx1.recv().await.unwrap();
        assert_eq!(20, even);
    }
}

pub trait GroupAggregate<I, T, U>
where
    I: Hash + Eq + PartialEq,
    T: IntoIterator<Item = I>,
{
    fn group_aggregate(t: T) -> U;
}

#[derive(Deserialize)]
pub struct GroupSumAggregatorConfig {}

impl FromPath for GroupSumAggregatorConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(GroupSumAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<GroupSumAggregator> for GroupSumAggregatorConfig {}

#[async_trait]
impl FromConfig<GroupSumAggregatorConfig> for GroupSumAggregator {
    async fn from_config(_config: &GroupSumAggregatorConfig) -> anyhow::Result<Self> {
        Ok(GroupSumAggregator {})
    }
}

pub struct GroupSumAggregator {}

impl<I, T> GroupAggregate<I, T, HashMap<I, I>> for GroupSumAggregator
where
    I: Hash + Eq + PartialEq + std::ops::AddAssign<I> + Clone,
    T: IntoIterator<Item = I>,
{
    fn group_aggregate(t: T) -> HashMap<I, I> {
        let mut group_sum: HashMap<I, I> = HashMap::new();
        for ref item in t {
            match group_sum.get_mut(item) {
                Some(sum) => *sum += item.to_owned(),
                None => {
                    group_sum.insert(item.to_owned(), item.to_owned());
                }
            };
        }
        group_sum
    }
}

#[async_trait]
impl<I, T> Map<T, HashMap<I, I>, GroupSumAggregatorConfig> for GroupSumAggregator
where
    I: Hash + Eq + PartialEq + std::ops::AddAssign<I> + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<HashMap<I, I>> {
        Ok(Self::group_aggregate(data))
    }
}

#[cfg(test)]
mod test_group_aggregator {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record(tx: Sender<Vec<u32>>, records: Vec<Vec<u32>>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_group_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1024);
        let (tx1, mut rx1) = channel!(HashMap<u32, u32>, 1024);
        let mut pipe = mapper!("group_summation", GroupSumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_record(tx0, vec![vec![2, 3, 2, 3, 2]]);
        f0.await;
        spawn_join!(pipe);
        let gs = rx1.recv().await.unwrap();
        assert_eq!(&6, gs.get(&2).unwrap());
        assert_eq!(&6, gs.get(&3).unwrap());
    }
}
