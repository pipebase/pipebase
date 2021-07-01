mod count;
mod pair;
mod sort;
mod sum;

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::FromIterator,
};

pub use count::*;
pub use pair::*;
pub use sort::*;
pub use sum::*;

pub trait Init {
    fn init() -> Self;
}

pub trait AggregateAs<T> {
    fn aggregate_value(&self) -> T;
}

pub trait Aggregate<I, T, U>
where
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> U;
}

pub trait GroupAs<T> {
    fn group_key(&self) -> T;
}

pub trait GroupTable<K, V>: IntoIterator<Item = (K, V)> {
    fn contains_group(&self, gid: &K) -> bool;
    fn insert_group(&mut self, gid: K, v: V) -> Option<V>;
    fn get_group_mut(&mut self, gid: &K) -> Option<&mut V>;
    fn get_group(&mut self, gid: &K) -> Option<&V>;
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

impl Init for u32 {
    fn init() -> u32 {
        0
    }
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

impl<K, V> GroupTable<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn contains_group(&self, gid: &K) -> bool {
        self.contains_key(gid)
    }

    fn get_group(&mut self, gid: &K) -> Option<&V> {
        self.get(gid)
    }

    fn get_group_mut(&mut self, gid: &K) -> Option<&mut V> {
        self.get_mut(gid)
    }

    fn insert_group(&mut self, gid: K, v: V) -> Option<V> {
        self.insert(gid, v)
    }
}

impl<K, V> GroupTable<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    fn contains_group(&self, gid: &K) -> bool {
        self.contains_key(gid)
    }

    fn get_group(&mut self, gid: &K) -> Option<&V> {
        self.get(gid)
    }

    fn get_group_mut(&mut self, gid: &K) -> Option<&mut V> {
        self.get_mut(gid)
    }

    fn insert_group(&mut self, gid: K, v: V) -> Option<V> {
        self.insert(gid, v)
    }
}
