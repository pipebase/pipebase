use serde::Deserialize;
use std::time::Duration;
use tokio::time::Interval;

use crate::{Collect, ConfigInto, FromConfig, FromFile};
use async_trait::async_trait;

#[derive(Deserialize)]
pub struct BagCollectorConfig {
    pub flush_period_in_millis: u64,
}

impl FromFile for BagCollectorConfig {}

#[async_trait]
impl<T> ConfigInto<BagCollector<T>> for BagCollectorConfig {}

pub struct BagCollector<T> {
    pub flush_period_in_millis: u64,
    pub buffer: Vec<T>,
}

#[async_trait]
impl<T> FromConfig<BagCollectorConfig> for BagCollector<T> {
    async fn from_config(config: &BagCollectorConfig) -> anyhow::Result<Self> {
        Ok(BagCollector {
            flush_period_in_millis: config.flush_period_in_millis,
            buffer: vec![],
        })
    }
}

#[async_trait]
impl<T: Clone + Send + Sync> Collect<T, Vec<T>, BagCollectorConfig> for BagCollector<T> {
    async fn collect(&mut self, t: &T) {
        self.buffer.push(t.to_owned())
    }

    async fn flush(&mut self) -> Vec<T> {
        let buffer = self.buffer.to_owned();
        self.buffer.clear();
        buffer
    }

    fn get_flush_interval(&self) -> Interval {
        tokio::time::interval(Duration::from_millis(self.flush_period_in_millis))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        channel, collector, context::State, spawn_join, BagCollectorConfig, Collector, FromFile,
        Pipe,
    };
    use tokio::sync::mpsc::{Receiver, Sender};

    #[derive(Clone, Debug)]
    struct Record {
        pub key: String,
        pub val: i32,
    }

    async fn populate_record(tx: Sender<Record>, records: Vec<Record>) {
        for r in records {
            let _ = tx.send(r).await;
        }
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
    async fn test_bag_collector() {
        let (tx0, rx0) = channel!(Record, 10);
        let (tx1, mut rx1) = channel!(Vec<Record>, 10);
        let mut pipe = collector!(
            "bag",
            "resources/catalogs/bag_collector.yml",
            BagCollectorConfig,
            rx0,
            [tx1]
        );
        let context = pipe.get_context();
        let ph = populate_record(
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
        spawn_join!(pipe);
        let records = receive_records(&mut rx1).await;
        assert_eq!(3, records.len());
        assert_eq!(0, records.get(0).unwrap().val);
        assert_eq!(1, records.get(1).unwrap().val);
        assert_eq!(2, records.get(2).unwrap().val);
        let context = context.read().await;
        assert_eq!(State::Done, context.get_state());
    }
}
