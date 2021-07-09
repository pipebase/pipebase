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

#[async_trait]
impl FromPath for SumAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
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

    #[tokio::test]
    async fn test_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(u32, 1024);
        let mut pipe = mapper!("summation");
        let f0 = populate_records(tx0, vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]]);
        f0.await;
        join_pipes!([run_pipe!(pipe, SumAggregatorConfig, [tx1], rx0)]);
        let odd = rx1.recv().await.unwrap();
        assert_eq!(16, odd);
        let even = rx1.recv().await.unwrap();
        assert_eq!(20, even);
    }

    #[derive(AggregateAs)]
    struct Record {
        #[agg(sum)]
        value: u32,
    }

    impl Record {
        pub fn new(value: u32) -> Self {
            Record { value: value }
        }
    }

    #[tokio::test]
    async fn test_record_sum() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(u32, 1024);
        let mut pipe = mapper!("record_sum");
        let f0 = populate_records(
            tx0,
            vec![vec![Record::new(1), Record::new(2), Record::new(3)]],
        );
        f0.await;
        let run_pipe = run_pipe!(pipe, SumAggregatorConfig, [tx1], rx0);
        let _ = run_pipe.await;
        let sum = rx1.recv().await.unwrap();
        assert_eq!(6, sum)
    }
}

#[derive(Deserialize)]
pub struct UnorderedGroupSumAggregatorConfig {}

#[async_trait]
impl FromPath for UnorderedGroupSumAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
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
    fn group_table(&self) -> G;
    fn group_aggregate(&self, t: T) -> U {
        let mut group_sum = self.group_table();
        for ref item in t {
            if !group_sum.contains_group(&item.group()) {
                group_sum.insert_group(item.group(), V::init());
            }
            let sum = group_sum.get_group_mut(&item.group()).unwrap();
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

// Group key is hashed but not ordered
pub struct UnorderedGroupSumAggregator {}

impl<I, T, K, V> GroupSumAggregate<I, T, K, V, Vec<Pair<K, V>>, HashMap<K, V>>
    for UnorderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    fn group_table(&self) -> HashMap<K, V> {
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

    #[tokio::test]
    async fn test_u32_group_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<u32, u32>>, 1024);
        let mut pipe = mapper!("group_summation");
        let f0 = populate_records(tx0, vec![vec![2, 3, 2, 3, 2, 3]]);
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            UnorderedGroupSumAggregatorConfig,
            [tx1],
            rx0
        )]);
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
        let mut pipe = mapper!("word_count");
        let f0 = populate_records(
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
        join_pipes!([run_pipe!(
            pipe,
            UnorderedGroupSumAggregatorConfig,
            [tx1],
            rx0
        )]);
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

    #[tokio::test]
    async fn test_record_group_sum() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, u32>>, 1024);
        let mut pipe = mapper!("record_sum");
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record::new("foo", 1),
                Record::new("foo", 2),
                Record::new("bar", 3),
            ]],
        );
        f0.await;
        let pipe_run = run_pipe!(pipe, UnorderedGroupSumAggregatorConfig, [tx1], rx0);
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
    }
}

#[derive(Deserialize)]
pub struct OrderedGroupSumAggregatorConfig {}

#[async_trait]
impl FromPath for OrderedGroupSumAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
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

// Group key is ordered
pub struct OrderedGroupSumAggregator {}

impl<I, T, K, V> GroupSumAggregate<I, T, K, V, Vec<Pair<K, V>>, BTreeMap<K, V>>
    for OrderedGroupSumAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Ord,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    fn group_table(&self) -> BTreeMap<K, V> {
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

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("ordered_word_count");
        let f0 = populate_records(
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
        join_pipes!([run_pipe!(pipe, OrderedGroupSumAggregatorConfig, [tx1], rx0)]);
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
