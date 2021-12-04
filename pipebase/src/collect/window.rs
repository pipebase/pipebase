use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::time::Interval;

use super::Collect;
use crate::common::{ConfigInto, FromConfig, FromPath, Period};
use async_trait::async_trait;

#[derive(Deserialize)]
pub struct InMemoryWindowCollectorConfig {
    size: Period,
    slice: Period,
}

impl FromPath for InMemoryWindowCollectorConfig {}

pub struct InstantContainer<T> {
    t: T,
    instant: Instant,
}

impl<T> InstantContainer<T> {
    pub fn new(t: T) -> Self {
        InstantContainer {
            t,
            instant: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub fn get(&self) -> &T {
        &self.t
    }
}

pub struct InMemoryWindowCollector<T> {
    size: Duration,
    slice: Duration,
    buffer: Vec<InstantContainer<T>>,
}

impl<T> ConfigInto<InMemoryWindowCollector<T>> for InMemoryWindowCollectorConfig {}

#[async_trait]
impl<T> FromConfig<InMemoryWindowCollectorConfig> for InMemoryWindowCollector<T> {
    async fn from_config(config: InMemoryWindowCollectorConfig) -> anyhow::Result<Self> {
        Ok(InMemoryWindowCollector {
            size: config.size.into(),
            slice: config.slice.into(),
            buffer: Vec::new(),
        })
    }
}

#[async_trait]
impl<T> Collect<T, Vec<T>, InMemoryWindowCollectorConfig> for InMemoryWindowCollector<T>
where
    T: Clone + Send + 'static,
{
    async fn collect(&mut self, t: T) -> anyhow::Result<()> {
        self.window_collect(t);
        Ok(())
    }

    async fn flush(&mut self) -> anyhow::Result<Option<Vec<T>>> {
        let items = self.flush_window();
        if items.is_empty() {
            return Ok(None);
        }
        Ok(Some(items))
    }

    fn get_flush_interval(&self) -> Interval {
        tokio::time::interval(self.slice.to_owned())
    }
}

impl<T> InMemoryWindowCollector<T>
where
    T: Clone,
{
    fn window_collect(&mut self, t: T) {
        self.buffer.push(InstantContainer::new(t))
    }

    fn flush_window(&mut self) -> Vec<T> {
        let mut cursor: usize = 0;
        for item in &self.buffer {
            let elapsed = item.elapsed();
            // no longer in window
            if elapsed > self.size {
                cursor += 1;
                continue;
            }
            break;
        }
        self.buffer.drain(0..cursor);
        self.buffer.iter().map(|item| item.get().clone()).collect()
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_window() {
        let (tx0, rx0) = channel!(u128, 1024);
        let (tx1, mut rx1) = channel!(Vec<u128>, 1024);
        let mut timer = poller!("timer");
        let mut window = collector!("window");
        let timer = run_pipe!(timer, TimerConfig, "resources/catalogs/timer.yml", [tx0]);
        let window = run_pipe!(
            window,
            InMemoryWindowCollectorConfig,
            "resources/catalogs/window.yml",
            [tx1],
            rx0
        );
        join_pipes!([timer, window]);
        let mut counts: HashMap<u128, usize> = HashMap::new();
        while let Some(ticks) = rx1.recv().await {
            for tick in ticks {
                *counts.entry(tick).or_insert(0) += 1;
            }
        }
        for i in 0..9 {
            let count = counts.get(&(i as u128)).unwrap();
            assert!(*count > 1 && *count <= 4)
        }
    }
}
