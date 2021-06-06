use crate::error::Result;
use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::Poll;

pub struct Timer {
    interval: Duration,
    ticks: u128,
}

#[async_trait]
impl Poll<Instant> for Timer {
    async fn poll(&mut self) -> Option<Result<Instant>> {
        match self.ticks > 0 {
            true => self.ticks -= 1,
            false => return None,
        }
        let mut interval = tokio::time::interval(self.interval);
        interval.tick().await;
        Some(Ok(Instant::now()))
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
        let ticks: u128 = 3;
        let mut s: Source<Instant> = Source::<Instant> {
            name: "timer",
            tx: tx,
            p: Box::new(Timer {
                interval: Duration::from_secs(1),
                ticks: ticks,
            }),
        };
        let f0 = s.run();
        let f1 = on_receive(&mut rx, ticks);
        tokio::join!(f0, f1);
    }
}
