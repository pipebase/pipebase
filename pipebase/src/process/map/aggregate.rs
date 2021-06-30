use std::{fmt::Debug, hash::Hash, ops::AddAssign};

use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

pub trait ZeroValue {
    fn zero_value() -> Self;
}

impl ZeroValue for u32 {
    fn zero_value() -> u32 {
        0
    }
}

pub trait AggregateAs<T>
where
    T: ZeroValue,
{
    fn aggregate_value(&self) -> T;
}

impl AggregateAs<u32> for u32 {
    fn aggregate_value(&self) -> u32 {
        *self
    }
}

pub trait Aggregate<I, T, U>
where
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
    U: ZeroValue,
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

impl<I, T, U> Aggregate<I, T, U> for SumAggregator
where
    I: AggregateAs<U>,
    U: std::ops::AddAssign<U> + ZeroValue + Default,
    T: IntoIterator<Item = I>,
{
    fn aggregate(t: T) -> U {
        let mut sum: U = U::zero_value();
        for item in t.into_iter() {
            sum += item.aggregate_value();
        }
        sum
    }
}

#[async_trait]
impl<I, T, U> Map<T, U, SumAggregatorConfig> for SumAggregator
where
    I: AggregateAs<U>,
    U: std::ops::AddAssign<U> + Default + ZeroValue,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<U> {
        Ok(Self::aggregate(data))
    }
}

#[cfg(test)]
mod aggregator_tests {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record(tx: Sender<Vec<u32>>, records: Vec<Vec<u32>>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(u32, 1024);
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

pub trait GroupAs<T>
where
    T: Hash + Eq + PartialEq,
{
    fn group_key(&self) -> T;
}

impl GroupAs<u32> for u32 {
    fn group_key(&self) -> u32 {
        *self
    }
}

pub trait GroupAggregate<I, T, K, V, U>
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    V: ZeroValue,
    U: IntoIterator<Item = Pair<K, V>>,
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

impl<I, T, K, V> GroupAggregate<I, T, K, V, Vec<Pair<K, V>>> for GroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + ZeroValue + Clone,
{
    fn group_aggregate(t: T) -> Vec<Pair<K, V>> {
        let mut group_sum: HashMap<K, V> = HashMap::new();
        for ref item in t {
            if !group_sum.contains_key(&item.group_key()) {
                group_sum.insert(item.group_key(), V::zero_value());
            }
            let sum = group_sum.get_mut(&item.group_key()).unwrap();
            *sum += item.aggregate_value();
        }
        group_sum.into_iter().map(|t| Pair::from(t)).collect()
    }
}

#[async_trait]
impl<I, T, K, V> Map<T, Vec<Pair<K, V>>, GroupSumAggregatorConfig> for GroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + ZeroValue + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
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
        let (tx1, mut rx1) = channel!(Vec<Pair<u32, u32>>, 1024);
        let mut pipe = mapper!("group_summation", GroupSumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_record(tx0, vec![vec![2, 3, 2, 3, 2, 3]]);
        f0.await;
        spawn_join!(pipe);
        let gs = rx1.recv().await.unwrap();
        for p in gs {
            match p.left() {
                &2 => assert_eq!(&6, p.right()),
                &3 => assert_eq!(&9, p.right()),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pair<K, V>(K, V);

impl<K, V> Pair<K, V> {
    pub fn new(k: K, v: V) -> Self {
        Pair(k, v)
    }

    pub fn left(&self) -> &K {
        &self.0
    }

    pub fn right(&self) -> &V {
        &self.1
    }
}

impl<K, V> From<(K, V)> for Pair<K, V> {
    fn from(t: (K, V)) -> Self {
        Pair(t.0, t.1)
    }
}

impl<K, V> GroupAs<K> for Pair<K, V>
where
    K: Clone + Hash + Eq + PartialEq,
{
    fn group_key(&self) -> K {
        self.0.to_owned()
    }
}

impl<K, V> AggregateAs<V> for Pair<K, V>
where
    V: ZeroValue + Clone,
{
    fn aggregate_value(&self) -> V {
        self.1.to_owned()
    }
}

impl<K, V> AddAssign<Self> for Pair<K, V>
where
    K: Eq + PartialEq + Debug,
    V: AddAssign<V>,
{
    fn add_assign(&mut self, rhs: Self) {
        if !self.0.eq(&rhs.0) {
            panic!(
                "can not add assign pair with different key: self {:?}, rhs {:?}",
                self.0, rhs.0
            );
        }
        self.1 += rhs.1
    }
}

#[cfg(test)]
mod pair_tests {

    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<Vec<Pair<String, u32>>>, records: Vec<Pair<String, u32>>) {
        let _ = tx.send(records).await;
    }

    #[tokio::test]
    async fn test_pair_group_sum() {
        let (tx0, rx0) = channel!(Vec<Pair<String, u32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, u32>>, 1024);
        let mut pipe = mapper!("pair_group_summation", GroupSumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_records(
            tx0,
            vec![
                Pair::new("foo".to_owned(), 1),
                Pair::new("foo".to_owned(), 2),
                Pair::new("bar".to_owned(), 2),
            ],
        );
        f0.await;
        spawn_join!(pipe);
        let gs = rx1.recv().await.unwrap();
        for p in gs {
            match p.left().as_str() {
                "foo" => assert_eq!(&3, p.right()),
                "bar" => assert_eq!(&2, p.right()),
                _ => unreachable!(),
            }
        }
    }
}
