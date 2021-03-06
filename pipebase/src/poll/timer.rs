use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use std::u128;

use super::{Poll, PollResponse};
use crate::common::{ConfigInto, FromConfig, FromPath, Period};
use tokio::time::Interval;

#[derive(Deserialize)]
pub struct TimerConfig {
    pub interval: Period,
    pub delay: Option<Period>,
    pub ticks: u128,
}

impl FromPath for TimerConfig {}

#[async_trait]
impl ConfigInto<Timer> for TimerConfig {}

/// Use tokio::time::Interval and emit tick in period
pub struct Timer {
    /// Interval between ticks
    pub interval: Duration,
    /// Initial delay
    pub delay: Duration,
    pub ticks: u128,
    pub tick: u128,
}

#[async_trait]
impl FromConfig<TimerConfig> for Timer {
    async fn from_config(config: TimerConfig) -> anyhow::Result<Timer> {
        let delay = match config.delay {
            Some(period) => period.into(),
            None => Duration::from_micros(0),
        };
        Ok(Timer {
            interval: config.interval.into(),
            delay,
            ticks: config.ticks,
            tick: 0,
        })
    }
}

/// # Parameters
/// * u128: output
#[async_trait]
impl Poll<u128, TimerConfig> for Timer {
    async fn poll(&mut self) -> anyhow::Result<PollResponse<u128>> {
        let tick = match self.ticks > 0 {
            true => self.tick,
            false => return Ok(PollResponse::Exit),
        };
        self.tick += 1;
        self.ticks -= 1;
        Ok(PollResponse::PollResult(Some(tick)))
    }

    fn get_initial_delay(&self) -> Duration {
        self.delay.to_owned()
    }

    fn get_interval(&self) -> Interval {
        let interval = self.interval.to_owned();
        tokio::time::interval(interval)
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;
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
        let channels = pipe_channels!([tx]);
        let config0 = config!(TimerConfig, "resources/catalogs/timer.yml");
        let config1 = config!(
            ContextPrinterConfig,
            "resources/catalogs/context_printer.yml"
        );
        let timer = poller!("timer");
        let ctx_printer = cstore!("ctx_printer");
        let ctx_collector = collect_context!([timer]);
        let run_ctx_printer = run_cstore!(ctx_printer, config1, ctx_collector);
        let run_timer = run_pipe!(timer, config0, channels);
        join_pipes!([run_timer, run_ctx_printer]);
        on_receive(&mut rx, 10).await;
    }

    #[tokio::test]
    async fn test_receiver_drop() {
        let (tx, rx) = channel!(u128, 1024);
        let channels = pipe_channels!([tx]);
        let config0 = config!(TimerConfig, "resources/catalogs/timer.yml");
        let source = poller!("timer");
        drop(rx);
        let start_millis = std::time::SystemTime::now();
        join_pipes!([run_pipe!(source, config0, channels)]);
        // poller should exit since receiver gone
        let now_millis = std::time::SystemTime::now();
        // poller should exit asap
        let duration = now_millis.duration_since(start_millis).unwrap();
        assert!(duration.as_secs() < 3)
    }
}
