use std::{cmp::Ordering, fmt::Debug, hash::Hash, ops::AddAssign};

use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq)]
pub struct Count32(u32);

impl Count32 {
    pub fn new(c: u32) -> Self {
        Count32(c)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl AddAssign<Self> for Count32 {
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

pub trait Init {
    fn init() -> Self;
}

impl Init for u32 {
    fn init() -> u32 {
        0
    }
}

impl Init for Count32 {
    fn init() -> Count32 {
        Count32(0)
    }
}

pub trait AggregateAs<T> {
    fn aggregate_value(&self) -> T;
}

impl AggregateAs<u32> for u32 {
    fn aggregate_value(&self) -> u32 {
        *self
    }
}

impl AggregateAs<Vec<u32>> for u32 {
    fn aggregate_value(&self) -> Vec<u32> {
        vec![*self]
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

pub trait Aggregate<I, T, U>
where
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> U;
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
pub struct TopAggregatorConfig {
    pub n: usize,
    pub desc: bool,
}

impl FromPath for TopAggregatorConfig {}

#[async_trait]
impl ConfigInto<TopAggregator> for TopAggregatorConfig {}

pub struct TopAggregator {
    n: usize,
    desc: bool,
}

impl TopAggregator {
    fn top<U>(v: &mut [U], n: usize, desc: bool)
    where
        U: Ord + Clone,
    {
        let n = n.min(v.len());
        let mut l = 0;
        let mut r = v.len() - 1;
        while l < r {
            let p = Self::partition(v, l, r, desc);
            if p + 1 == n {
                return;
            }
            if p + 1 > n {
                r = p - 1;
                continue;
            }
            l = p + 1
        }
    }

    fn partition<U>(v: &mut [U], l: usize, mut r: usize, desc: bool) -> usize
    where
        U: Ord + Clone,
    {
        let pivot = v[l].to_owned();
        let mut j = l;
        let mut i = l + 1;
        while i <= r {
            let u = v[i].to_owned();
            if (u >= pivot && desc) || (u <= pivot && !desc) {
                j += 1;
                v.swap(i, j);
                i += 1;
                continue;
            }
            v.swap(i, r);
            r -= 1;
        }
        v.swap(l, j);
        j
    }
}

#[async_trait]
impl FromConfig<TopAggregatorConfig> for TopAggregator {
    async fn from_config(config: &TopAggregatorConfig) -> anyhow::Result<Self> {
        Ok(TopAggregator {
            n: config.n,
            desc: config.desc,
        })
    }
}

impl<I, T, U> Aggregate<I, T, Vec<U>> for TopAggregator
where
    I: AggregateAs<Vec<U>>,
    U: Ord + Clone,
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> Vec<U> {
        let mut buffer: Vec<U> = Vec::new();
        for item in t {
            buffer.extend(item.aggregate_value())
        }
        // apply n and desc
        Self::top(&mut buffer, self.n, self.desc);
        let n = self.n;
        while buffer.len() > n {
            buffer.pop();
        }
        buffer.sort_by(|a, b| match self.desc {
            true => b.partial_cmp(a).unwrap(),
            false => a.partial_cmp(b).unwrap(),
        });
        buffer
    }
}

#[async_trait]
impl<I, T, U> Map<T, Vec<U>, TopAggregatorConfig> for TopAggregator
where
    I: AggregateAs<Vec<U>>,
    U: Ord + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<U>> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod top_aggregator_tests {

    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record(tx: Sender<Vec<u32>>, records: Vec<Vec<u32>>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_top_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(Vec<u32>, 1024);
        let mut pipe = mapper!(
            "top",
            "resources/catalogs/top_aggregator.yml",
            TopAggregatorConfig,
            rx0,
            [tx1]
        );
        let f0 = populate_record(
            tx0,
            vec![vec![1, 2, 2, 3], vec![1, 1, 2, 1], vec![2, 2, 2, 2]],
        );
        f0.await;
        spawn_join!(pipe);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![3, 2, 2], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 1, 1], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 2, 2], r);
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

impl GroupAs<String> for String {
    fn group_key(&self) -> String {
        self.to_owned()
    }
}

pub trait GroupAggregate<I, T, K, V, U>
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    U: IntoIterator<Item = Pair<K, V>>,
{
    fn group_aggregate(&self, t: T) -> U;
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
    V: std::ops::AddAssign<V> + Init + Clone,
{
    fn group_aggregate(&self, t: T) -> Vec<Pair<K, V>> {
        let mut group_sum: HashMap<K, V> = HashMap::new();
        for ref item in t {
            if !group_sum.contains_key(&item.group_key()) {
                group_sum.insert(item.group_key(), V::init());
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

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("word_count", GroupSumAggregatorConfig, rx0, [tx1]);
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
    V: Clone,
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
