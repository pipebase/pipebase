use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::{
    Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath, GroupAs, GroupTable, Init, Map, Pair,
};
use async_trait::async_trait;
use serde::Deserialize;

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
    U: std::ops::AddAssign<U> + Init,
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> U {
        let mut sum: U = U::init();
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
    U: std::ops::AddAssign<U> + Default + Init,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<U> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod sum_aggregator_tests {
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

#[derive(Deserialize)]
pub struct UnorderedGroupSumAggregatorConfig {}

impl FromPath for UnorderedGroupSumAggregatorConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(UnorderedGroupSumAggregatorConfig {})
    }
}

pub trait GroupSumAggregate<I, T, K, V, U, G>
where
    I: GroupAs<K> + AggregateAs<V>,
    V: std::ops::AddAssign<V> + Init + Clone,
    T: IntoIterator<Item = I>,
    U: FromIterator<Pair<K, V>>,
    G: GroupTable<K, V>,
{
    fn new_group_table(&self) -> G;
    fn group_aggregate(&self, t: T) -> U {
        let mut group_sum = self.new_group_table();
        for ref item in t {
            if !group_sum.contains_group(&item.group_key()) {
                group_sum.insert_group(item.group_key(), V::init());
            }
            let sum = group_sum.get_group_mut(&item.group_key()).unwrap();
            *sum += item.aggregate_value();
        }
        group_sum.into_iter().map(|t| Pair::from(t)).collect()
    }
}

#[async_trait]
impl ConfigInto<UnorderedGroupSumAggregator> for UnorderedGroupSumAggregatorConfig {}

#[async_trait]
impl FromConfig<UnorderedGroupSumAggregatorConfig> for UnorderedGroupSumAggregator {
    async fn from_config(_config: &UnorderedGroupSumAggregatorConfig) -> anyhow::Result<Self> {
        Ok(UnorderedGroupSumAggregator {})
    }
}

pub struct UnorderedGroupSumAggregator {}

impl<I, T, K, V> GroupSumAggregate<I, T, K, V, Vec<Pair<K, V>>, HashMap<K, V>>
    for UnorderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    fn new_group_table(&self) -> HashMap<K, V> {
        HashMap::new()
    }
}

#[async_trait]
impl<I, T, K, V> Map<T, Vec<Pair<K, V>>, UnorderedGroupSumAggregatorConfig>
    for UnorderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + Init + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        Ok(self.group_aggregate(data))
    }
}

#[cfg(test)]
mod test_group_aggregator {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record<T>(tx: Sender<T>, records: Vec<T>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_u32_group_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<u32, u32>>, 1024);
        let mut pipe = mapper!(
            "group_summation",
            UnorderedGroupSumAggregatorConfig,
            rx0,
            [tx1]
        );
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

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("word_count", UnorderedGroupSumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_record(
            tx0,
            vec![vec![
                "foo".to_owned(),
                "foo".to_owned(),
                "bar".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
            ]],
        );
        f0.await;
        spawn_join!(pipe);
        let wcs = rx2.recv().await.unwrap();
        for wc in wcs {
            match wc.left().as_str() {
                "foo" => assert_eq!(2, wc.right().get()),
                "bar" => assert_eq!(1, wc.right().get()),
                "buz" => assert_eq!(3, wc.right().get()),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct OrderedGroupSumAggregatorConfig {}

impl FromPath for OrderedGroupSumAggregatorConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(OrderedGroupSumAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<OrderedGroupSumAggregator> for OrderedGroupSumAggregatorConfig {}

#[async_trait]
impl FromConfig<OrderedGroupSumAggregatorConfig> for OrderedGroupSumAggregator {
    async fn from_config(_config: &OrderedGroupSumAggregatorConfig) -> anyhow::Result<Self> {
        Ok(OrderedGroupSumAggregator {})
    }
}

pub struct OrderedGroupSumAggregator {}

impl<I, T, K, V> GroupSumAggregate<I, T, K, V, Vec<Pair<K, V>>, BTreeMap<K, V>>
    for OrderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Ord,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    fn new_group_table(&self) -> BTreeMap<K, V> {
        BTreeMap::new()
    }
}

#[async_trait]
impl<I, T, K, V> Map<T, Vec<Pair<K, V>>, OrderedGroupSumAggregatorConfig>
    for OrderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Ord,
    V: std::ops::AddAssign<V> + Init + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        Ok(self.group_aggregate(data))
    }
}

#[cfg(test)]
mod test_ordered_group_aggregator {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record<T>(tx: Sender<T>, records: Vec<T>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!(
            "ordered_word_count",
            OrderedGroupSumAggregatorConfig,
            rx0,
            [tx1]
        );
        let f0 = populate_record(
            tx0,
            vec![vec![
                "foo".to_owned(),
                "foo".to_owned(),
                "bar".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
            ]],
        );
        f0.await;
        spawn_join!(pipe);
        let wcs = rx2.recv().await.unwrap();
        let mut wcs_iter = wcs.into_iter();
        let bar = wcs_iter.next().unwrap();
        assert_eq!("bar", bar.left());
        assert_eq!(1, bar.right().get());
        let buz = wcs_iter.next().unwrap();
        assert_eq!("buz", buz.left());
        assert_eq!(3, buz.right().get());
        let foo = wcs_iter.next().unwrap();
        assert_eq!("foo", foo.left());
        assert_eq!(2, foo.right().get());
    }
}
