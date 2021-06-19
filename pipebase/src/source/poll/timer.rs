use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use std::time::Duration;
use std::u128;
use tokio::time::Interval;

use crate::{ConfigInto, FromConfig, FromFile, Poll};

#[derive(Deserialize)]
pub struct TimerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}

impl FromFile for TimerConfig {}

#[async_trait]
impl ConfigInto<Timer> for TimerConfig {}

pub struct Timer {
    pub interval: Interval,
    pub ticks: u128,
    pub tick: u128,
}

#[async_trait]
impl FromConfig<TimerConfig> for Timer {
    async fn from_config(
        config: &TimerConfig,
    ) -> std::result::Result<Timer, Box<dyn std::error::Error>> {
        Ok(Timer {
            interval: tokio::time::interval(Duration::from_millis(config.period_in_millis)),
            ticks: config.ticks,
            tick: 0,
        })
    }
}

#[async_trait]
impl Poll<u128, TimerConfig> for Timer {
    async fn poll(&mut self) -> Result<Option<u128>, Box<dyn Error + Send + Sync>> {
        self.interval.tick().await;
        let tick = match self.ticks > 0 {
            true => {
                let tick = self.tick;
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

    use crate::{channel, poller, spawn_join, FromFile, Pipe, Poller, TimerConfig};
    use tokio::sync::mpsc::Receiver;

    async fn on_receive(rx: &mut Receiver<u128>, ticks: u128) {
        let mut i = 0;
        while ticks > i {
            rx.recv().await.unwrap();
            println!("tick: #{:#?}", i);
            i += 1;
        }
    }

    #[tokio::test]
    async fn test_timer() {
        let (tx, mut rx) = channel!(u128, 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimerConfig,
            dummy, // dummy receiver ignored
            [tx]
        );
        spawn_join!(source);
        on_receive(&mut rx, 10).await;
    }

    #[tokio::test]
    async fn test_receiver_drop() {
        let (tx, rx) = channel!(u128, 1024);
        let mut source = poller!("timer", "resources/catalogs/timer.yml", TimerConfig, [tx]);
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
