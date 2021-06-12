use serde::Deserialize;
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time::Interval;

use crate::{Collect, ConfigInto, FromConfig, FromFile};
use async_trait::async_trait;

#[derive(Deserialize)]
pub struct SetCollectorConfig {
    pub flush_period_in_millis: u64,
}

impl FromFile for SetCollectorConfig {}

#[async_trait]
impl<T: Ord> ConfigInto<SetCollector<T>> for SetCollectorConfig {}

pub struct SetCollector<T: Ord> {
    pub flush_period_in_millis: u64,
    pub set: BTreeSet<T>,
    pub buffer: Vec<T>,
}

#[async_trait]
impl<T: Ord> FromConfig<SetCollectorConfig> for SetCollector<T> {
    async fn from_config(
        config: &SetCollectorConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(SetCollector {
            flush_period_in_millis: config.flush_period_in_millis,
            set: BTreeSet::new(),
            buffer: vec![],
        })
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + Ord> Collect<T, SetCollectorConfig> for SetCollector<T> {
    async fn collect(&mut self, t: &T) {
        match self.set.insert(t.to_owned()) {
            true => self.buffer.push(t.to_owned()),
            false => (),
        }
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
        channel, collector, context::State, spawn_join, Collector, FromFile, Pipe,
        SetCollectorConfig,
    };
    use std::cmp::Ordering;
    use tokio::sync::mpsc::{channel, Receiver, Sender};

    #[derive(Clone, Debug, Eq)]
    struct Record {
        pub key: String,
        pub val: i32,
    }

    impl Ord for Record {
        fn cmp(&self, other: &Self) -> Ordering {
            self.key.cmp(&other.key)
        }
    }

    impl PartialOrd for Record {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for Record {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key
        }
    }

    async fn populate_record(tx: Sender<Record>, records: Vec<Record>) {
        for r in records {
            tx.send(r).await;
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
            "resources/catalogs/set_collector.yml",
            SetCollectorConfig,
            rx0,
            [tx1]
        );
        let context = pipe.get_context();
        let ph = populate_record(
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
        spawn_join!(pipe);
        let records = receive_records(&mut rx1).await;
        assert_eq!(1, records.len());
        assert_eq!(0, records.get(0).unwrap().val);
        let context = context.read().await;
        assert_eq!(State::Done, context.get_state());
    }
}
