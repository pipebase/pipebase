use serde::Deserialize;
use std::collections::HashSet;
use std::hash::Hash;
use tokio::time::Interval;

use super::Collect;
use crate::common::{ConfigInto, FromConfig, FromPath, Period, Set};
use async_trait::async_trait;

/// Collect unique item
#[async_trait]
pub trait SetCollect<T, S>
where
    T: Send + 'static,
    S: Set<T> + Send,
{
    fn get_set(&mut self) -> &mut S;

    /// Collect item
    async fn set_collect(&mut self, t: T) -> anyhow::Result<()> {
        let set = self.get_set();
        set.collect(t).await
    }

    /// Flush set and return items
    async fn flush_set(&mut self) -> anyhow::Result<Vec<T>> {
        let set = self.get_set();
        let set = set.flush().await?;
        Ok(set)
    }
}

#[derive(Deserialize)]
pub struct InMemorySetCollectorConfig {
    pub flush_period: Period,
}

impl FromPath for InMemorySetCollectorConfig {}

#[async_trait]
impl<T> ConfigInto<InMemorySetCollector<T>> for InMemorySetCollectorConfig {}

/// In memory cache unique items
pub struct InMemorySetCollector<T> {
    /// Caller should flush cache every flush_period
    pub flush_period: Period,
    pub buffer: HashSet<T>,
}

#[async_trait]
impl<T> FromConfig<InMemorySetCollectorConfig> for InMemorySetCollector<T> {
    async fn from_config(config: InMemorySetCollectorConfig) -> anyhow::Result<Self> {
        Ok(InMemorySetCollector {
            flush_period: config.flush_period,
            buffer: HashSet::new(),
        })
    }
}

#[async_trait]
impl<T> SetCollect<T, HashSet<T>> for InMemorySetCollector<T>
where
    T: Hash + Eq + Clone + Send + 'static,
{
    fn get_set(&mut self) -> &mut HashSet<T> {
        &mut self.buffer
    }
}

/// # Parameters
/// * T: input
/// * Vec<T>: output
#[async_trait]
impl<T> Collect<T, Vec<T>, InMemorySetCollectorConfig> for InMemorySetCollector<T>
where
    T: Clone + Send + Hash + Eq + 'static,
{
    async fn collect(&mut self, t: T) -> anyhow::Result<()> {
        self.set_collect(t).await
    }

    async fn flush(&mut self) -> anyhow::Result<Option<Vec<T>>> {
        let set = self.flush_set().await?;
        if set.is_empty() {
            return Ok(None);
        }
        return Ok(Some(set));
    }

    /// Call by collector pipe to flush set in period
    fn get_flush_interval(&self) -> Interval {
        let flush_period = self.flush_period.clone();
        tokio::time::interval(flush_period.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Clone, Debug, Eq, HashedBy, Equal)]
    struct Record {
        #[hash]
        #[equal]
        pub key: String,
        pub val: i32,
    }

    #[tokio::test]
    async fn test_set_collector() {
        let (tx0, rx0) = channel!(Record, 10);
        let (tx1, mut rx1) = channel!(Vec<Record>, 10);
        let channels = pipe_channels!(rx0, [tx1]);
        let pipe = collector!("set_collector");
        let context = pipe.get_context();
        let ph = populate_records(
            tx0,
            vec![
                Record {
                    key: "1".to_owned(),
                    val: 0,
                },
                Record {
                    key: "1".to_owned(),
                    val: 1,
                },
                Record {
                    key: "1".to_owned(),
                    val: 2,
                },
            ],
        );
        ph.await;
        join_pipes!([run_pipe!(
            pipe,
            InMemorySetCollectorConfig,
            "resources/catalogs/set_collector.yml",
            channels
        )]);
        let records = rx1.recv().await.unwrap();
        assert_eq!(1, records.len());
        assert_eq!(0, records.get(0).unwrap().val);
        assert_eq!(State::Done, context.get_state());
    }
}
