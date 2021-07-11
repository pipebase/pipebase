mod count;
mod pair;
mod sort;
mod sum;

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::{FromIterator, IntoIterator},
};

pub use count::*;
pub use pair::*;
pub use sort::*;
pub use sum::*;

pub trait Init {
    fn init() -> Self;
}

impl<T> Init for Vec<T> {
    fn init() -> Self {
        Vec::new()
    }
}

pub trait AggregateAs<T> {
    fn aggregate_value(&self) -> T;
}

pub trait Aggregate<I, T, U>
where
    U: Init,
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
{
    fn merge(&self, u: &mut U, i: &I);

    // post merge operation
    fn operate(&self, u: U) -> U {
        u
    }

    fn aggregate(&self, t: T) -> U {
        let mut u = U::init();
        for i in t {
            self.merge(&mut u, &i);
        }
        self.operate(u)
    }
}

pub trait GroupAs<T> {
    fn group(&self) -> T;
}

pub trait GroupTable<K, V>: IntoIterator<Item = (K, V)> {
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool>;
    fn insert_group(&mut self, gid: K, v: V) -> anyhow::Result<Option<V>>;
    fn get_group(&mut self, gid: &K) -> anyhow::Result<Option<&mut V>>;
    fn persist_groups(&mut self) -> anyhow::Result<()> {
        Ok(())
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
    fn group(&self) -> u32 {
        *self
    }
}

impl GroupAs<String> for String {
    fn group(&self) -> String {
        self.to_owned()
    }
}

impl<K, V> GroupTable<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool> {
        Ok(self.contains_key(gid))
    }

    fn get_group(&mut self, gid: &K) -> anyhow::Result<Option<&mut V>> {
        Ok(self.get_mut(gid))
    }

    fn insert_group(&mut self, gid: K, v: V) -> anyhow::Result<Option<V>> {
        Ok(self.insert(gid, v))
    }
}

impl<K, V> GroupTable<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool> {
        Ok(self.contains_key(gid))
    }

    fn get_group(&mut self, gid: &K) -> anyhow::Result<Option<&mut V>> {
        Ok(self.get_mut(gid))
    }

    fn insert_group(&mut self, gid: K, v: V) -> anyhow::Result<Option<V>> {
        Ok(self.insert(gid, v))
    }
}

pub trait GroupAggregate<I, T, K, V, U, G>
where
    I: GroupAs<K> + AggregateAs<V>,
    V: Init + Clone,
    T: IntoIterator<Item = I>,
    U: FromIterator<Pair<K, V>>,
    G: GroupTable<K, V>,
{
    fn merge(&self, v: &mut V, i: &I);
    fn group_table(&self) -> anyhow::Result<G>;
    fn group_aggregate(&self, t: T) -> anyhow::Result<U> {
        let mut group_table = self.group_table()?;
        for ref item in t {
            if !group_table.contains_group(&item.group())? {
                group_table.insert_group(item.group(), V::init())?;
            }
            let v = group_table
                .get_group(&item.group())?
                .expect("group not found");
            self.merge(v, item);
        }
        // persist aggregated groups
        group_table.persist_groups()?;
        Ok(group_table.into_iter().map(|t| Pair::from(t)).collect())
    }
}
