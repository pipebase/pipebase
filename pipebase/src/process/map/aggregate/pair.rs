use crate::{AggregateAs, GroupAs};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt::Debug,
    hash::Hash,
};

pub trait LeftRight<L, R> {
    fn left(&self) -> &L;
    fn right(&self) -> &R;
}

// General Pair
#[derive(Debug, Clone)]
pub struct Pair<L, R>(L, R);

impl<L, R> Pair<L, R> {
    pub fn new(k: L, v: R) -> Self {
        Pair(k, v)
    }
}

impl<L, R> LeftRight<L, R> for Pair<L, R> {
    fn left(&self) -> &L {
        &self.0
    }

    fn right(&self) -> &R {
        &self.1
    }
}

impl<L, R> From<(L, R)> for Pair<L, R> {
    fn from(t: (L, R)) -> Self {
        Pair(t.0, t.1)
    }
}

// Pair's left as group key
// Pair's right is a comparable and aggregable value
#[derive(Debug, Clone, Eq)]
pub struct RightValuedPair<L, R>(L, R);

impl<L, R> RightValuedPair<L, R> {
    pub fn new(k: L, v: R) -> Self {
        RightValuedPair(k, v)
    }
}

impl<L, R> LeftRight<L, R> for RightValuedPair<L, R> {
    fn left(&self) -> &L {
        &self.0
    }

    fn right(&self) -> &R {
        &self.1
    }
}

impl<L, R> Ord for RightValuedPair<L, R>
where
    L: Eq,
    R: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<L, R> PartialOrd for RightValuedPair<L, R>
where
    L: Eq,
    R: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<L, R> PartialEq for RightValuedPair<L, R>
where
    R: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<L, R> From<(L, R)> for RightValuedPair<L, R> {
    fn from(t: (L, R)) -> Self {
        RightValuedPair(t.0, t.1)
    }
}

impl<L, R> GroupAs<L> for RightValuedPair<L, R>
where
    L: Clone + Hash + Eq + PartialEq,
{
    fn group(&self) -> L {
        self.0.to_owned()
    }
}

impl<L, R> AggregateAs<R> for RightValuedPair<L, R>
where
    R: Clone,
{
    fn aggregate_value(&self) -> R {
        self.1.to_owned()
    }
}

impl<L, R> AggregateAs<Vec<RightValuedPair<L, R>>> for RightValuedPair<L, R>
where
    L: Clone,
    R: Clone,
{
    fn aggregate_value(&self) -> Vec<RightValuedPair<L, R>> {
        vec![self.to_owned()]
    }
}

impl<L, R> std::ops::AddAssign<Self> for RightValuedPair<L, R>
where
    L: Eq + PartialEq + Debug,
    R: std::ops::AddAssign<R>,
{
    fn add_assign(&mut self, rhs: Self) {
        if !self.0.eq(&rhs.0) {
            panic!(
                "can not add assign pair with different left: self {:?}, rhs {:?}",
                self.0, rhs.0
            );
        }
        self.1 += rhs.1
    }
}

#[cfg(test)]
mod pair_tests {

    #[test]
    fn test_right_ordered_pair_cmp() {
        let p0 = RightValuedPair::new("foo".to_owned(), 1);
        let p1 = RightValuedPair::new("foo".to_owned(), 2);
        assert!(p0 < p1);
        let p2 = RightValuedPair::new("bar".to_owned(), 2);
        assert_eq!(p1, p2);
        assert!(p0 < p2);
    }

    use crate::*;

    #[tokio::test]
    async fn test_right_ordered_pair_group_sum() {
        let (tx0, rx0) = channel!(Vec<RightValuedPair<String, u32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, u32>>, 1024);
        let mut pipe = mapper!("pair_group_summation");
        let f0 = populate_records(
            tx0,
            vec![vec![
                RightValuedPair::new("foo".to_owned(), 1),
                RightValuedPair::new("foo".to_owned(), 2),
                RightValuedPair::new("bar".to_owned(), 2),
            ]],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            UnorderedGroupSumAggregatorConfig,
            [tx1],
            rx0
        )]);
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
        let (tx0, rx0) = channel!(Vec<RightValuedPair<String, Count32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<RightValuedPair<String, Count32>>, 1024);
        let mut pipe = Mapper::new("top_word");
        let f0 = populate_records(
            tx0,
            vec![vec![
                RightValuedPair::new("d".to_owned(), Count32::new(4)),
                RightValuedPair::new("a".to_owned(), Count32::new(1)),
                RightValuedPair::new("e".to_owned(), Count32::new(5)),
                RightValuedPair::new("b".to_owned(), Count32::new(2)),
                RightValuedPair::new("c".to_owned(), Count32::new(3)),
            ]],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            TopAggregatorConfig,
            "resources/catalogs/top_aggregator_desc.yml",
            [tx1],
            rx0
        )]);
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
