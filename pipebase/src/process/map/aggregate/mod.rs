mod count;
mod pair;
mod sort;
mod sum;

pub use count::*;
pub use pair::*;
pub use sort::*;
pub use sum::*;

use std::hash::Hash;

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

pub trait GroupAggregate<I, T, K, V, U>
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Hash + Eq + PartialEq,
    U: IntoIterator<Item = Pair<K, V>>,
{
    fn group_aggregate(&self, t: T) -> U;
}

pub trait OrderedGroupAggregate<I, T, K, V, U>
where
    I: GroupAs<K> + AggregateAs<V>,
    T: IntoIterator<Item = I>,
    K: Ord,
    U: IntoIterator<Item = Pair<K, V>>,
{
    fn group_aggregate(&self, t: T) -> U;
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
