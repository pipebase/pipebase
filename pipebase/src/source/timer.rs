use async_trait::async_trait;
use std::error::Error;
use std::result::Result;
use std::time::Duration;
use tokio::time::Interval;

use crate::Poll;

pub struct TimerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}
pub struct Timer {
    pub interval: Interval,
    pub ticks: u128,
}

impl From<TimerConfig> for Timer {
    fn from(config: TimerConfig) -> Self {
        Timer {
            interval: tokio::time::interval(Duration::from_millis(config.period_in_millis)),
            ticks: config.ticks,
        }
    }
}

#[async_trait]
impl Poll<()> for Timer {
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
    use crate::source::timer::TimerConfig;

    use super::super::Source;
    use super::Timer;
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
    async fn test_timer() {
        let (tx, mut rx) = channel::<()>(1024);
        let ticks = 10;
        let config = TimerConfig {
            period_in_millis: 1000,
            ticks: ticks,
        };
        let mut s: Source<()> = Source::<()> {
            name: "timer",
            txs: vec![tx],
            poller: Box::new(Timer::from(config)),
        };
        let f0 = s.run();
        let f1 = on_receive(&mut rx, ticks);
        tokio::join!(f0, f1);
    }
}
