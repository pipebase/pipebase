use serde::Deserialize;
use tokio::time::Interval;

use super::Collect;
use crate::common::{Bag, ConfigInto, FromConfig, FromPath, Period};
use async_trait::async_trait;

/// Collect items
#[async_trait]
pub trait BagCollect<T, B>
where
    T: Send + 'static,
    B: Bag<T> + Send,
{
    fn get_bag(&mut self) -> &mut B;

    /// Collect item
    async fn bag_collect(&mut self, t: T) -> anyhow::Result<()> {
        let b = self.get_bag();
        b.collect(t).await
    }

    /// Flush bag and return items
    async fn flush_bag(&mut self) -> anyhow::Result<Vec<T>> {
        let bag = self.get_bag();
        let bag = bag.flush().await?;
        Ok(bag)
    }
}

#[derive(Deserialize)]
pub struct InMemoryBagCollectorConfig {
    pub flush_period: Period,
}

impl FromPath for InMemoryBagCollectorConfig {}

#[async_trait]
impl<T> ConfigInto<InMemoryBagCollector<T>> for InMemoryBagCollectorConfig {}

/// In memory cache items
pub struct InMemoryBagCollector<T> {
    /// Caller should flush cache every flush_period
    pub flush_period: Period,
    pub buffer: Vec<T>,
}

#[async_trait]
impl<T> FromConfig<InMemoryBagCollectorConfig> for InMemoryBagCollector<T> {
    async fn from_config(config: InMemoryBagCollectorConfig) -> anyhow::Result<Self> {
        Ok(InMemoryBagCollector {
            flush_period: config.flush_period,
            buffer: vec![],
        })
    }
}

#[async_trait]
impl<T> BagCollect<T, Vec<T>> for InMemoryBagCollector<T>
where
    T: Clone + Send + 'static,
{
    fn get_bag(&mut self) -> &mut Vec<T> {
        &mut self.buffer
    }
}

/// # Parameters
/// * T: input
/// * Vec<T>: output
#[async_trait]
impl<T> Collect<T, Vec<T>, InMemoryBagCollectorConfig> for InMemoryBagCollector<T>
where
    T: Clone + Send + 'static,
{
    async fn collect(&mut self, t: T) -> anyhow::Result<()> {
        self.bag_collect(t).await
    }

    async fn flush(&mut self) -> anyhow::Result<Option<Vec<T>>> {
        let bag = self.flush_bag().await?;
        if bag.is_empty() {
            return Ok(None);
        }
        return Ok(Some(bag));
    }

    /// Call by collector pipe to flush bag in period
    fn get_flush_interval(&self) -> Interval {
        let flush_period = self.flush_period.clone();
        tokio::time::interval(flush_period.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use tokio::sync::mpsc::Receiver;

    #[derive(Clone, Debug)]
    struct Record {
        pub key: String,
        pub val: i32,
    }

    async fn receive_records(rx: &mut Receiver<Vec<Record>>) -> Vec<Record> {
        let mut all_records: Vec<Record> = vec![];
        loop {
            match rx.recv().await {
                Some(records) => all_records.extend(records),
                None => return all_records,
            }
        }
    }

    #[tokio::test]
    async fn test_in_mem_bag_collector() {
        let (tx0, rx0) = channel!(Record, 10);
        let (tx1, mut rx1) = channel!(Vec<Record>, 10);
        let pipe = collector!("bag_collector");
        let context = pipe.get_context();
        let ph = populate_records(
            tx0,
            vec![
                Record {
                    key: "0".to_owned(),
                    val: 0,
                },
                Record {
                    key: "1".to_owned(),
                    val: 1,
                },
                Record {
                    key: "2".to_owned(),
                    val: 2,
                },
            ],
        );
        ph.await;
        join_pipes!([run_pipe!(
            pipe,
            InMemoryBagCollectorConfig,
            "resources/catalogs/bag_collector.yml",
            [tx1],
            rx0
        )]);
        let records = receive_records(&mut rx1).await;
        assert_eq!(3, records.len());
        assert_eq!(0, records.get(0).unwrap().val);
        assert_eq!(1, records.get(1).unwrap().val);
        assert_eq!(2, records.get(2).unwrap().val);
        assert_eq!(State::Done, context.get_state());
    }

    #[tokio::test]
    async fn test_collector_exit() {
        let (tx0, rx0) = channel!(u128, 1024);
        let (tx1, rx1) = channel!(Vec<u128>, 1024);
        let timer = poller!("timer");
        let collector = collector!("tick_collector");
        let run_timer = run_pipe!(timer, TimerConfig, "resources/catalogs/timer.yml", [tx0]);
        let run_collector = run_pipe!(
            collector,
            InMemoryBagCollectorConfig,
            "resources/catalogs/bag_collector.yml",
            [tx1],
            rx0
        );
        let start_millis = std::time::SystemTime::now();
        drop(rx1);
        join_pipes!([run_timer, run_collector]);
        let now_millis = std::time::SystemTime::now();
        // timer and collector should exit asap since downstream rx1 dropped
        let duration = now_millis.duration_since(start_millis).unwrap();
        assert!(duration.as_secs() < 10)
    }
}
