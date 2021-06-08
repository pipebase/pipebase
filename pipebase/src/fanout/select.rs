use crate::FromConfig;
use async_trait::async_trait;
use rand::{prelude::ThreadRng, Rng};
use std::{sync::Arc, usize};
pub trait Select: Send + Sync {
    fn select(&mut self) -> Vec<usize>;
    fn get_range(&mut self) -> usize;
}

pub struct RandomConfig {
    pub n: usize,
}

pub struct Random {
    n: usize,
}

#[async_trait]
impl FromConfig<RandomConfig> for Random {
    async fn from_config(
        config: &RandomConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Random { n: config.n })
    }
}

impl Select for Random {
    fn select(&mut self) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.n);
        vec![i]
    }

    fn get_range(&mut self) -> usize {
        self.n
    }
}

pub struct RoundRobinConfig {
    pub n: usize,
}

pub struct RoundRobin {
    pub i: usize,
    pub n: usize,
}

#[async_trait]
impl FromConfig<RoundRobinConfig> for RoundRobin {
    async fn from_config(
        config: &RoundRobinConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(RoundRobin { i: 0, n: config.n })
    }
}

impl Select for RoundRobin {
    fn select(&mut self) -> Vec<usize> {
        let i = self.i;
        let selected = vec![i];
        self.i = (i + 1) % self.n;
        selected
    }
    fn get_range(&mut self) -> usize {
        self.n
    }
}

pub struct BroadcastConfig {
    pub n: usize,
}

pub struct Broadcast {
    n: usize,
}

#[async_trait]
impl FromConfig<BroadcastConfig> for Broadcast {
    async fn from_config(
        config: &BroadcastConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Broadcast { n: config.n })
    }
}

impl Select for Broadcast {
    fn select(&mut self) -> Vec<usize> {
        (0..self.n).collect()
    }
    fn get_range(&mut self) -> usize {
        self.n
    }
}

#[cfg(test)]
mod tests {
    use super::super::Selector;
    use super::*;
    use crate::{Source, Timer, TimerConfig};
    use tokio::sync::mpsc::{channel, Receiver};
    use tokio::task::JoinHandle;

    async fn count_tick(rx: &mut Receiver<()>, id: usize) -> usize {
        let mut c: usize = 0;
        loop {
            match rx.recv().await {
                Some(()) => {
                    c += 1;
                    println!("id: {}, ticks: {}", id, c);
                }
                None => return c,
            }
        }
    }

    #[tokio::test]
    async fn test_random_select() {
        let (tx0, rx0) = channel::<()>(1024);
        let (tx1, mut rx1) = channel::<()>(1024);
        let (tx2, mut rx2) = channel::<()>(1024);
        let selector_txs = vec![tx1, tx2];
        let timer_tx = tx0;
        let ticks = 10;
        let timer_config = TimerConfig {
            period_in_millis: 1000,
            ticks: ticks,
        };
        let timer = Timer::from_config(&timer_config).await.unwrap();
        let mut source = Source {
            name: "timer",
            tx: timer_tx,
            poller: Box::new(timer),
        };
        let random_selector_config = RandomConfig { n: 2 };
        let random_selector = Random::from_config(&random_selector_config).await.unwrap();
        let mut selector = Selector {
            name: "random_select",
            rx: rx0,
            txs: selector_txs,
            selector: Box::new(random_selector),
        };
        let join_src = tokio::spawn(async move {
            source.run().await;
        });
        let join_selector = tokio::spawn(async move {
            match selector.run().await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("error from selector {}", e)
                }
            }
        });
        tokio::join!(join_src, join_selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(ticks as usize, c1 + c2);
    }
}
