use serde::Deserialize;
use std::collections::HashSet;
use std::hash::Hash;
use std::time::Duration;
use tokio::time::Interval;

use super::Collect;
use crate::common::{ConfigInto, FromConfig, FromPath, Set};
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
    async fn set_collect(&mut self, t: T) {
        let set = self.get_set();
        set.collect(t).await;
    }

    /// Flush set and return items
    async fn flush_set(&mut self) -> Vec<T> {
        let set = self.get_set();
        set.flush().await
    }
}

#[derive(Deserialize)]
pub struct InMemorySetCollectorConfig {
    pub flush_period_in_millis: u64,
}

impl FromPath for InMemorySetCollectorConfig {}

#[async_trait]
impl<T> ConfigInto<InMemorySetCollector<T>> for InMemorySetCollectorConfig {}

/// In memory cache unique items
pub struct InMemorySetCollector<T> {
    /// Caller should flush cache every flush_period millis
    pub flush_period_in_millis: u64,
    pub buffer: HashSet<T>,
}

#[async_trait]
impl<T> FromConfig<InMemorySetCollectorConfig> for InMemorySetCollector<T> {
    async fn from_config(config: InMemorySetCollectorConfig) -> anyhow::Result<Self> {
        Ok(InMemorySetCollector {
            flush_period_in_millis: config.flush_period_in_millis,
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
    async fn collect(&mut self, t: T) {
        self.set_collect(t).await;
    }

    async fn flush(&mut self) -> Vec<T> {
        self.flush_set().await
    }

    /// Call by collector pipe to flush set in period
    fn get_flush_interval(&self) -> Interval {
        tokio::time::interval(Duration::from_millis(self.flush_period_in_millis))
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
        let mut pipe = collector!("set_collector");
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
            [tx1],
            rx0
        )]);
        let records = rx1.recv().await.unwrap();
        assert_eq!(1, records.len());
        assert_eq!(0, records.get(0).unwrap().val);
        assert_eq!(State::Done, context.get_state());
    }
}