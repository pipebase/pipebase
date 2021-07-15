use crate::{AggregateAs, Init, Pair};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::iter::{FromIterator, IntoIterator};

pub trait GroupAs<T> {
    fn group(&self) -> T;
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

pub trait GroupTable<K, V>: IntoIterator<Item = (K, V)> {
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool>;
    fn insert_group(&mut self, gid: K, v: V) -> anyhow::Result<Option<V>>;
    fn get_group(&mut self, gid: &K) -> anyhow::Result<Option<&mut V>>;
    fn persist_groups(&mut self) -> anyhow::Result<()> {
        Ok(())
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
    fn operate(&self, _v: &mut V) {
        return;
    }
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
        Ok(group_table
            .into_iter()
            .map(|mut t| {
                self.operate(&mut t.1);
                Pair::from(t)
            })
            .collect())
    }
}
