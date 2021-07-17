use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use std::u128;
use tokio::time::{sleep, Interval};

use crate::{ConfigInto, FromConfig, FromPath, Poll};

#[derive(Clone, Deserialize)]
pub enum Period {
    Millis(u64),
    Secs(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
}

impl From<Period> for Duration {
    fn from(period: Period) -> Self {
        match period {
            Period::Millis(m) => Duration::from_millis(m),
            Period::Secs(s) => Duration::from_secs(s),
            Period::Minutes(m) => Duration::from_secs(m * 60),
            Period::Hours(h) => Duration::from_secs(h * 3600),
            Period::Days(d) => Duration::from_secs(d * 3600 * 3600),
        }
    }
}

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
    pub interval: Interval,
    /// Initial delay
    pub delay: Duration,
    pub ticks: u128,
    pub tick: u128,
}

#[async_trait]
impl FromConfig<TimerConfig> for Timer {
    async fn from_config(config: &TimerConfig) -> anyhow::Result<Timer> {
        let delay = match config.delay {
            Some(ref period) => period.to_owned().into(),
            None => Duration::from_micros(0),
        };
        let interval = config.interval.to_owned();
        Ok(Timer {
            interval: tokio::time::interval(interval.into()),
            delay: delay,
            ticks: config.ticks,
            tick: 0,
        })
    }
}

/// # Parameters
/// * u128: Output
#[async_trait]
impl Poll<u128, TimerConfig> for Timer {
    async fn poll(&mut self) -> anyhow::Result<Option<u128>> {
        let tick = match self.ticks > 0 {
            true => self.tick,
            false => return Ok(None),
        };
        if tick == 0 {
            // apply initial deplay
            sleep(self.delay).await;
        }
        self.interval.tick().await;
        self.tick += 1;
        self.ticks -= 1;
        Ok(Some(tick))
    }
}

#[cfg(test)]
mod tests {

    use crate::*;
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
        let mut timer = Poller::new("timer");
        let mut ctx_printer = cstore!("ctx_printer");
        let run_ctx_printer = run_cstore!(
            ctx_printer,
            ContextPrinterConfig,
            "resources/catalogs/context_printer.yml",
            [timer]
        );
        let run_timer = run_pipe!(timer, TimerConfig, "resources/catalogs/timer.yml", [tx]);
        join_pipes!([run_timer, run_ctx_printer]);
        on_receive(&mut rx, 10).await;
    }

    #[tokio::test]
    async fn test_receiver_drop() {
        let (tx, rx) = channel!(u128, 1024);
        let mut source = poller!("timer");
        drop(rx);
        let start_millis = std::time::SystemTime::now();
        join_pipes!([run_pipe!(
            source,
            TimerConfig,
            "resources/catalogs/timer.yml",
            [tx]
        )]);
        // poller should exit since receiver gone
        let now_millis = std::time::SystemTime::now();
        // poller should exit asap
        let duration = now_millis.duration_since(start_millis).unwrap();
        assert!(duration.as_secs() < 3)
    }
}
