use serde::Deserialize;
use std::time::Duration;
use tokio::time::Interval;

use crate::{Collect, ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;

#[async_trait]
pub trait Bag<T> {
    async fn collect(&mut self, t: T);
    async fn flush(&mut self) -> Vec<T>;
}

#[async_trait]
impl<T> Bag<T> for Vec<T>
where
    T: Clone + Send,
{
    async fn collect(&mut self, t: T) {
        self.push(t);
    }

    async fn flush(&mut self) -> Vec<T> {
        let buffer = self.to_owned();
        self.clear();
        buffer
    }
}

#[async_trait]
pub trait BagCollect<T, B>
where
    T: Send + 'static,
    B: Bag<T> + Send,
{
    fn get_bag(&mut self) -> &mut B;

    async fn bag_collect(&mut self, t: T) {
        let b = self.get_bag();
        b.collect(t).await;
    }

    async fn flush_bag(&mut self) -> Vec<T> {
        let b = self.get_bag();
        b.flush().await
    }
}

#[derive(Deserialize)]
pub struct InMemoryBagCollectorConfig {
    pub flush_period_in_millis: u64,
}

impl FromPath for InMemoryBagCollectorConfig {}

#[async_trait]
impl<T> ConfigInto<InMemoryBagCollector<T>> for InMemoryBagCollectorConfig {}

pub struct InMemoryBagCollector<T> {
    pub flush_period_in_millis: u64,
    pub buffer: Vec<T>,
}

#[async_trait]
impl<T> FromConfig<InMemoryBagCollectorConfig> for InMemoryBagCollector<T> {
    async fn from_config(config: &InMemoryBagCollectorConfig) -> anyhow::Result<Self> {
        Ok(InMemoryBagCollector {
            flush_period_in_millis: config.flush_period_in_millis,
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

#[async_trait]
impl<T> Collect<T, Vec<T>, InMemoryBagCollectorConfig> for InMemoryBagCollector<T>
where
    T: Clone + Send + 'static,
{
    async fn collect(&mut self, t: T) {
        self.bag_collect(t).await
    }

    async fn flush(&mut self) -> Vec<T> {
        self.flush_bag().await
    }

    fn get_flush_interval(&self) -> Interval {
        tokio::time::interval(Duration::from_millis(self.flush_period_in_millis))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
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
        let mut pipe = collector!("bag_collector");
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
        let context = context.read().await;
        assert_eq!(State::Done, context.get_state());
    }
}
