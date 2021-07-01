use crate::{AggregateAs, GroupAs};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt::Debug,
    hash::Hash,
};

#[derive(Debug, Clone, Eq)]
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

    pub fn swap(&self) -> Pair<V, K>
    where
        K: Clone,
        V: Clone,
    {
        Pair(self.1.to_owned(), self.0.to_owned())
    }
}

impl<K, V> Ord for Pair<K, V>
where
    K: Eq,
    V: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<K, V> PartialOrd for Pair<K, V>
where
    K: Eq,
    V: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K, V> PartialEq for Pair<K, V>
where
    V: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
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

impl<K, V> AggregateAs<Vec<Pair<K, V>>> for Pair<K, V>
where
    K: Clone,
    V: Clone,
{
    fn aggregate_value(&self) -> Vec<Pair<K, V>> {
        vec![self.to_owned()]
    }
}

impl<K, V> std::ops::AddAssign<Self> for Pair<K, V>
where
    K: Eq + PartialEq + Debug,
    V: std::ops::AddAssign<V>,
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

    async fn populate_records<T>(tx: Sender<T>, records: T) {
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

    #[tokio::test]
    async fn test_top_pair() {
        let (tx0, rx0) = channel!(Vec<Pair<String, Count32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!(
            "top_word",
            "resources/catalogs/top_aggregator.yml",
            TopAggregatorConfig,
            rx0,
            [tx1]
        );
        let f0 = populate_records(
            tx0,
            vec![
                Pair::new("d".to_owned(), Count32::new(4)),
                Pair::new("a".to_owned(), Count32::new(1)),
                Pair::new("e".to_owned(), Count32::new(5)),
                Pair::new("b".to_owned(), Count32::new(2)),
                Pair::new("c".to_owned(), Count32::new(3)),
            ],
        );
        f0.await;
        spawn_join!(pipe);
        let top = rx1.recv().await.unwrap();
        assert_eq!(3, top.len());
        let top1 = top.get(0).unwrap();
        assert_eq!(5, top1.right().get());
        assert_eq!("e", top1.left());
        let top2 = top.get(1).unwrap();
        assert_eq!(4, top2.right().get());
        assert_eq!("d", top2.left());
        let top3 = top.get(2).unwrap();
        assert_eq!(3, top3.right().get());
        assert_eq!("c", top3.left());
    }
}
