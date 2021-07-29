use super::{AggregateAs, GroupAs};
use crate::common::Convert;
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt::Debug,
    hash::Hash,
};

pub trait LeftRight {
    type L;
    type R;
    fn left(&self) -> &Self::L;
    fn right(&self) -> &Self::R;
    fn as_tuple(self) -> (Self::L, Self::R);
}

impl<L, R> LeftRight for (L, R) {
    type L = L;
    type R = R;

    fn left(&self) -> &Self::L {
        &self.0
    }

    fn right(&self) -> &Self::R {
        &self.1
    }

    fn as_tuple(self) -> (L, R) {
        self
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Pair<L, R>(pub L, pub R);

impl<L, R> Pair<L, R> {
    pub fn new(k: L, v: R) -> Self {
        Pair(k, v)
    }
}

impl<L, R> LeftRight for Pair<L, R> {
    type L = L;
    type R = R;
    fn left(&self) -> &L {
        &self.0
    }

    fn right(&self) -> &R {
        &self.1
    }

    fn as_tuple(self) -> (Self::L, Self::R) {
        (self.0, self.1)
    }
}

impl<L, R> From<(L, R)> for Pair<L, R> {
    fn from(t: (L, R)) -> Self {
        Pair::new(t.0, t.1)
    }
}

impl<L, R, P> Convert<P> for Pair<L, R>
where
    P: LeftRight<L = L, R = R>,
    L: Clone,
    R: Clone,
{
    fn convert(rhs: P) -> Self {
        let (l, r) = rhs.as_tuple();
        Pair::new(l, r)
    }
}

impl<L, R> Ord for Pair<L, R>
where
    L: Eq,
    R: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<L, R> PartialOrd for Pair<L, R>
where
    L: Eq,
    R: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<L, R> PartialEq for Pair<L, R>
where
    R: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<L, R> GroupAs<L> for Pair<L, R>
where
    L: Clone + Hash + Eq + PartialEq,
{
    fn group(&self) -> L {
        self.0.to_owned()
    }
}

impl<L, R> AggregateAs<R> for Pair<L, R>
where
    R: Clone,
{
    fn aggregate_value(&self) -> R {
        self.1.to_owned()
    }
}

impl<L, R> AggregateAs<Vec<Pair<L, R>>> for Pair<L, R>
where
    L: Clone,
    R: Clone,
{
    fn aggregate_value(&self) -> Vec<Pair<L, R>> {
        vec![self.to_owned()]
    }
}

impl<L, R> std::ops::AddAssign<Self> for Pair<L, R>
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

impl<L, R> Hash for Pair<L, R>
where
    L: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[cfg(test)]
mod left_right_tests {

    use crate::prelude::*;

    #[derive(LeftRight)]
    struct Record {
        #[left]
        id: String,
        #[right]
        val: i32,
    }

    #[test]
    fn test_left_right() {
        let r = Record {
            id: "foo".to_owned(),
            val: 1,
        };
        assert_eq!("foo", r.left());
        assert_eq!(&1, r.right());
    }
}

#[cfg(test)]
mod pair_tests {

    use crate::prelude::*;

    #[test]
    fn test_right_ordered_pair_cmp() {
        let p0 = Pair::new("foo".to_owned(), 1);
        let p1 = Pair::new("foo".to_owned(), 2);
        assert!(p0 < p1);
        let p2 = Pair::new("bar".to_owned(), 2);
        assert_eq!(p1, p2);
        assert!(p0 < p2);
    }

    use crate::*;

    #[tokio::test]
    async fn test_right_ordered_pair_group_sum() {
        let (tx0, rx0) = channel!(Vec<Pair<String, u32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, u32>>, 1024);
        let mut pipe = mapper!("pair_group_summation");
        let f0 = populate_records(
            tx0,
            vec![vec![
                Pair::new("foo".to_owned(), 1),
                Pair::new("foo".to_owned(), 2),
                Pair::new("bar".to_owned(), 2),
            ]],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            UnorderedGroupAddAggregatorConfig,
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
        let (tx0, rx0) = channel!(Vec<Pair<String, Count32>>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = Mapper::new("top_word");
        let f0 = populate_records(
            tx0,
            vec![vec![
                Pair::new("d".to_owned(), Count32::new(4)),
                Pair::new("a".to_owned(), Count32::new(1)),
                Pair::new("e".to_owned(), Count32::new(5)),
                Pair::new("b".to_owned(), Count32::new(2)),
                Pair::new("c".to_owned(), Count32::new(3)),
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
