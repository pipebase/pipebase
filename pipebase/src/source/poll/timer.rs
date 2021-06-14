use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::result::Result;
use std::time::Duration;
use std::u128;
use tokio::time::Interval;

use crate::{ConfigInto, FromConfig, FromFile, Poll};

#[derive(Clone, Debug)]
pub struct TimePollerTick {
    pub tick: u128,
}

impl Display for TimePollerTick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n  tick: {}\n}}", self.tick)
    }
}

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
    pub tick: u128,
}

#[async_trait]
impl FromConfig<TimePollerConfig> for TimePoller {
    async fn from_config(
        config: &TimePollerConfig,
    ) -> std::result::Result<TimePoller, Box<dyn std::error::Error>> {
        Ok(TimePoller {
            interval: tokio::time::interval(Duration::from_millis(config.period_in_millis)),
            ticks: config.ticks,
            tick: 0,
        })
    }
}

#[async_trait]
impl Poll<TimePollerTick, TimePollerConfig> for TimePoller {
    async fn poll(&mut self) -> Result<Option<TimePollerTick>, Box<dyn Error + Send + Sync>> {
        self.interval.tick().await;
        let tick = match self.ticks > 0 {
            true => {
                let tick = TimePollerTick { tick: self.tick };
                self.tick += 1;
                self.ticks -= 1;
                tick
            }
            false => return Ok(None),
        };
        Ok(Some(tick))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        channel, poller, spawn_join, FromFile, Pipe, Poller, TimePollerConfig, TimePollerTick,
    };
    use tokio::sync::mpsc::{channel, Receiver};

    async fn on_receive(rx: &mut Receiver<TimePollerTick>, ticks: u128) {
        let mut i = 0;
        while ticks > i {
            rx.recv().await.unwrap();
            println!("tick: #{:#?}", i);
            i += 1;
        }
    }

    #[tokio::test]
    async fn test_time_poller() {
        let (tx, mut rx) = channel!(TimePollerTick, 1024);
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
        let (tx, rx) = channel!(TimePollerTick, 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimePollerConfig,
            [tx]
        );
        drop(rx);
        let start_millis = std::time::SystemTime::now();
        spawn_join!(source);
        // poller should exit since receiver gone
        let now_millis = std::time::SystemTime::now();
        // poller should exit asap
        let duration = now_millis.duration_since(start_millis).unwrap();
        assert!(duration.as_secs() < 3)
    }
}
