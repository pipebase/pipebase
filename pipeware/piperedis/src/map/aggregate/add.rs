use super::table::RedisGroupTable;
use async_trait::async_trait;
use pipebase::{
    AggregateAs, ConfigInto, FromConfig, FromPath, GroupAggregate, GroupAs, Init, Map, Pair,
};
use redis::{FromRedisValue, ToRedisArgs};
use serde::Deserialize;
use std::collections::HashMap;
use std::{hash::Hash, iter::FromIterator};

#[derive(Deserialize)]
pub struct RedisUnorderedGroupAddAggregatorConfig {
    url: String,
}

impl FromPath for RedisUnorderedGroupAddAggregatorConfig {}

impl ConfigInto<RedisUnorderedGroupAddAggregator> for RedisUnorderedGroupAddAggregatorConfig {}

pub struct RedisUnorderedGroupAddAggregator {
    url: String,
}

#[async_trait]
impl FromConfig<RedisUnorderedGroupAddAggregatorConfig> for RedisUnorderedGroupAddAggregator {
    async fn from_config(config: &RedisUnorderedGroupAddAggregatorConfig) -> anyhow::Result<Self> {
        Ok(RedisUnorderedGroupAddAggregator {
            url: config.url.to_owned(),
        })
    }
}

impl<I, T, K, V, U> GroupAggregate<I, T, K, V, U, RedisGroupTable<HashMap<K, V>>>
    for RedisUnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + ToRedisArgs,
    V: std::ops::AddAssign<V> + Init + ToRedisArgs + FromRedisValue + Clone,
    T: IntoIterator<Item = I>,
    U: FromIterator<Pair<K, V>>,
{
    fn merge(&self, v: &mut V, i: &I) {
        *v += i.aggregate_value();
    }

    fn group_table(&self) -> anyhow::Result<RedisGroupTable<HashMap<K, V>>> {
        RedisGroupTable::new(&self.url, HashMap::new())
    }
}

#[async_trait]
impl<I, K, V, T> Map<T, Vec<Pair<K, V>>, RedisUnorderedGroupAddAggregatorConfig>
    for RedisUnorderedGroupAddAggregator
where
    I: GroupAs<K> + AggregateAs<V>,
    K: Hash + Eq + PartialEq + Clone + ToRedisArgs,
    V: std::ops::AddAssign<V> + Init + ToRedisArgs + FromRedisValue + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<Pair<K, V>>> {
        let sums = self.group_aggregate(data)?;
        Ok(sums)
    }
}
