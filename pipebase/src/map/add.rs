use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use super::Map;
use crate::common::{
    Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Init, Pair,
};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddAggregatorConfig {}

#[async_trait]
impl FromPath for AddAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(AddAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<AddAggregator> for AddAggregatorConfig {}

/// Aggregate items with `+=`
pub struct AddAggregator {}

#[async_trait]
impl FromConfig<AddAggregatorConfig> for AddAggregator {
    async fn from_config(_config: AddAggregatorConfig) -> anyhow::Result<Self> {
        Ok(AddAggregator {})
    }
}

impl<I, T, U> Aggregate<I, T, U> for AddAggregator
where
    I: AggregateAs<U>,
    U: std::ops::AddAssign<U> + Init,
    T: IntoIterator<Item = I>,
{
    /// Merge items of I as U using `+=`
    fn merge(&self, u: &mut U, i: &I) {
        *u += i.aggregate_value();
    }
}

#[async_trait]
impl<I, T, U> Map<T, U, AddAggregatorConfig> for AddAggregator
where
    I: AggregateAs<U>,
    U: std::ops::AddAssign<U> + Init,
    T: IntoIterator<Item = I> + Send + 'static,
{
    /// Input: T
    /// Output: U
    async fn map(&mut self, data: T) -> anyhow::Result<U> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod sum_aggregator_tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(u32, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(AddAggregatorConfig);
        let pipe = mapper!("summation");
        let f0 = populate_records(tx0, vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]]);
        f0.await;
        join_pipes!([run_pipe!(pipe, config, channels)]);
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
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(AddAggregatorConfig);
        let pipe = mapper!("record_sum");
        let f0 = populate_records(
            tx0,
            vec![vec![Record::new(1), Record::new(2), Record::new(3)]],
        );
        f0.await;
        let run_pipe = run_pipe!(pipe, config, channels);
        let _ = run_pipe.await;
        let sum = rx1.recv().await.unwrap();
        assert_eq!(6, sum)
    }
}

#[cfg(test)]
mod count32_tests {

    use crate::prelude::*;

    #[derive(Debug, Clone, AggregateAs)]
    #[agg(count32)]
    struct Record {}

    #[tokio::test]
    async fn test_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Count32, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(AddAggregatorConfig);
        let pipe = mapper!("counter");
        let pipe = run_pipe!(pipe, config, channels);
        let f0 = populate_records(tx0, vec![vec![Record {}, Record {}, Record {}, Record {}]]);
        f0.await;
        join_pipes!([pipe]);
        let c = rx1.recv().await.expect("count32 not found");
        assert_eq!(4, c.get())
    }
}

#[cfg(test)]
mod test_avg {

    use crate::prelude::*;

    #[derive(Clone, Debug, AggregateAs)]
    struct Record {
        id: String,
        #[agg(avgf32)]
        value: i32,
    }

    #[tokio::test]
    async fn test_averagef32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Averagef32, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(AddAggregatorConfig);
        let pipe = mapper!("average");
        let pipe = run_pipe!(pipe, config, channels);
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record {
                    id: "a".to_owned(),
                    value: 1,
                },
                Record {
                    id: "a".to_owned(),
                    value: 2,
                },
                Record {
                    id: "a".to_owned(),
                    value: 3,
                },
            ]],
        );
        f0.await;
        join_pipes!([pipe]);
        let avg = rx1.recv().await.expect("not average received");
        assert_eq!(2.0, avg.average())
    }
}

#[derive(Deserialize)]
pub struct UnorderedGroupAddAggregatorConfig {}

#[async_trait]
impl FromPath for UnorderedGroupAddAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(UnorderedGroupAddAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<UnorderedGroupAddAggregator> for UnorderedGroupAddAggregatorConfig {}

#[async_trait]
impl FromConfig<UnorderedGroupAddAggregatorConfig> for UnorderedGroupAddAggregator {
    async fn from_config(_config: UnorderedGroupAddAggregatorConfig) -> anyhow::Result<Self> {
        Ok(UnorderedGroupAddAggregator {})
    }
}

/// Group added result by key
pub struct UnorderedGroupAddAggregator {}

impl<I, T, K, V> GroupAggregate<I, T, K, V, Vec<Pair<K, V>>, HashMap<K, V>>
    for UnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    /// Merge items per group using `+=`
    fn merge(&self, v: &mut V, i: &I) {
        *v += i.aggregate_value();
    }

    fn group_table(&self) -> anyhow::Result<HashMap<K, V>> {
        Ok(HashMap::new())
    }
}

#[async_trait]
impl<I, T, K, V> Map<T, Vec<Pair<K, V>>, UnorderedGroupAddAggregatorConfig>
    for UnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq,
    V: std::ops::AddAssign<V> + Init + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    /// Input: T
    /// Output: Vec<Pair<K, V>>
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        Ok(self.group_aggregate(data)?)
    }
}

#[cfg(test)]
mod test_group_sum_aggregator {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_u32_group_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<u32, u32>>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(UnorderedGroupAddAggregatorConfig);
        let pipe = mapper!("group_summation");
        let f0 = populate_records(tx0, vec![vec![2, 3, 2, 3, 2, 3]]);
        f0.await;
        join_pipes!([run_pipe!(pipe, config, channels)]);
        let gs = rx1.recv().await.unwrap();
        for p in gs {
            match p.left() {
                &2 => assert_eq!(&6, p.right()),
                &3 => assert_eq!(&9, p.right()),
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
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(UnorderedGroupAddAggregatorConfig);
        let pipe = mapper!("record_sum");
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record::new("foo", 1),
                Record::new("foo", 2),
                Record::new("bar", 3),
            ]],
        );
        f0.await;
        let pipe_run = run_pipe!(pipe, config, channels);
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

#[cfg(test)]
mod unordered_group_avg_f32_tests {

    use crate::prelude::*;

    #[derive(Clone, Debug, AggregateAs, GroupAs)]
    struct Record {
        #[group]
        id: String,
        #[agg(avgf32)]
        value: i32,
    }

    #[tokio::test]
    async fn test_unordered_group_avg_f32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Averagef32>>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(UnorderedGroupAddAggregatorConfig);
        let pipe = mapper!("group_avg_f32");
        let pipe = run_pipe!(pipe, config, channels);
        let f0 = populate_records(
            tx0,
            vec![vec![
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
            ]],
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
    }
}

#[cfg(test)]
mod group_count32_tests {

    use crate::prelude::*;

    #[derive(Debug, Clone, GroupAs, AggregateAs)]
    #[agg(count32)]
    struct Record {
        #[group]
        key: String,
    }

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(UnorderedGroupAddAggregatorConfig);
        let pipe = mapper!("word_count");
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
        join_pipes!([run_pipe!(pipe, config, channels)]);
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

    #[tokio::test]
    async fn test_record_group_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(UnorderedGroupAddAggregatorConfig);
        let pipe = mapper!("group_count32");
        let pipe = run_pipe!(pipe, config, channels);
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

#[derive(Deserialize)]
pub struct OrderedGroupAddAggregatorConfig {}

#[async_trait]
impl FromPath for OrderedGroupAddAggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(OrderedGroupAddAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<OrderedGroupAddAggregator> for OrderedGroupAddAggregatorConfig {}

#[async_trait]
impl FromConfig<OrderedGroupAddAggregatorConfig> for OrderedGroupAddAggregator {
    async fn from_config(_config: OrderedGroupAddAggregatorConfig) -> anyhow::Result<Self> {
        Ok(OrderedGroupAddAggregator {})
    }
}

/// Group added result by ordered key
pub struct OrderedGroupAddAggregator {}

impl<I, T, K, V> GroupAggregate<I, T, K, V, Vec<Pair<K, V>>, BTreeMap<K, V>>
    for OrderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Ord,
    V: std::ops::AddAssign<V> + Init + Clone,
{
    /// Merge items per group using `+=`
    fn merge(&self, v: &mut V, i: &I) {
        *v += i.aggregate_value();
    }

    fn group_table(&self) -> anyhow::Result<BTreeMap<K, V>> {
        Ok(BTreeMap::new())
    }
}

/// # Parameters
/// * T: input
/// * Vec<Pair<K, V>>: output
/// * K: group value
/// * V: aggregate value
#[async_trait]
impl<I, T, K, V> Map<T, Vec<Pair<K, V>>, OrderedGroupAddAggregatorConfig>
    for OrderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Ord,
    V: std::ops::AddAssign<V> + Init + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        Ok(self.group_aggregate(data)?)
    }
}

#[cfg(test)]
mod test_ordered_group_aggregator {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(OrderedGroupAddAggregatorConfig);
        let pipe = mapper!("ordered_word_count");
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
        join_pipes!([run_pipe!(pipe, config, channels)]);
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
