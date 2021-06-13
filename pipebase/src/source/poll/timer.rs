use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use std::time::Duration;
use tokio::time::Interval;

use crate::{ConfigInto, FromConfig, FromFile, Poll};

#[derive(Deserialize)]
pub struct TimePollerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}

impl FromFile for TimePollerConfig {}

#[async_trait]
impl ConfigInto<TimePoller> for TimePollerConfig {}

pub struct TimePoller {
    pub interval: Interval,
    pub ticks: u128,
}

#[async_trait]
impl FromConfig<TimePollerConfig> for TimePoller {
    async fn from_config(
        config: &TimePollerConfig,
    ) -> std::result::Result<TimePoller, Box<dyn std::error::Error>> {
        Ok(TimePoller {
            interval: tokio::time::interval(Duration::from_millis(config.period_in_millis)),
            ticks: config.ticks,
        })
    }
}

#[async_trait]
impl Poll<(), TimePollerConfig> for TimePoller {
    async fn poll(&mut self) -> Result<Option<()>, Box<dyn Error + Send + Sync>> {
        match self.ticks > 0 {
            true => self.ticks -= 1,
            false => return Ok(None),
        }
        self.interval.tick().await;
        Ok(Some(()))
    }
}

#[cfg(test)]
mod tests {

    use crate::{channel, poller, spawn_join, FromFile, Pipe, Poller, TimePollerConfig};
    use tokio::sync::mpsc::{channel, Receiver};

    async fn on_receive(rx: &mut Receiver<()>, ticks: u128) {
        let mut i = 0;
        while ticks > i {
            rx.recv().await.unwrap();
            println!("tick: #{:#?}", i);
            i += 1;
        }
    }

    #[tokio::test]
    async fn test_time_poller() {
        let (tx, mut rx) = channel!((), 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimePollerConfig,
            [tx]
        );
        spawn_join!(source);
        on_receive(&mut rx, 10).await;
    }

    #[tokio::test]
    async fn test_receiver_drop() {
        let (tx, rx) = channel!((), 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimePollerConfig,
            [tx]
        );
        drop(rx);
        spawn_join!(source);
        // poller should exit since receiver gone
    }
}
