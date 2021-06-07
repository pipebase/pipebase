use async_trait::async_trait;
use std::error::Error;
use std::result::Result;
use std::time::{Duration, Instant};
use tokio::time::Interval;

use crate::Poll;

pub struct Timer {
    pub interval: Interval,
    pub ticks: u128,
}

#[async_trait]
impl Poll<Instant> for Timer {
    async fn poll(&mut self) -> Result<Option<Instant>, Box<dyn Error + Send + Sync>> {
        match self.ticks > 0 {
            true => self.ticks -= 1,
            false => return Ok(None),
        }
        self.interval.tick().await;
        Ok(Some(Instant::now()))
    }
}

impl Timer {
    pub fn new(interval: Duration, ticks: u128) -> Timer {
        Timer {
            interval: tokio::time::interval(interval),
            ticks: ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Source;
    use super::Timer;
    use std::time::{Duration, Instant};
    use tokio::sync::mpsc::{channel, Receiver};

    async fn on_receive(rx: &mut Receiver<Instant>, ticks: u128) {
        let mut ticks = ticks;
        while ticks > 0 {
            let timestamp = rx.recv().await.unwrap();
            println!("timestamp: {:#?}", timestamp);
            ticks -= 1;
        }
    }

    #[tokio::test]
    async fn test_timer() {
        let (tx, mut rx) = channel::<Instant>(1024);
        let ticks: u128 = 10;
        let mut s: Source<Instant> = Source::<Instant> {
            name: "timer",
            txs: vec![tx],
            poller: Box::new(Timer::new(Duration::from_secs(1), ticks)),
        };
        let f0 = s.run();
        let f1 = on_receive(&mut rx, ticks);
        tokio::join!(f0, f1);
    }
}
