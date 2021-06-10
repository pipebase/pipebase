use crate::{FromConfig, FromFile};
use async_trait::async_trait;
use rand::Rng;
use serde::Deserialize;

pub trait Select: Send {
    fn select(&mut self) -> Vec<usize>;
    fn get_range(&mut self) -> usize;
}

#[derive(Deserialize)]
pub struct RandomConfig {
    pub n: usize,
}

impl FromFile for RandomConfig {}

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
#[macro_use]
mod tests {

    use super::super::Selector;
    use super::*;
    use crate::selector;
    use crate::source;
    use crate::{Source, Timer, TimerConfig};
    use tokio::sync::mpsc::{channel, Receiver};

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
        let mut source = source!(
            "timer",
            "resources/catalogs/timer.yml",
            TimerConfig,
            Timer,
            [tx0]
        );
        let mut selector = selector!(
            "random_select",
            "resources/catalogs/random_selector.yml",
            RandomConfig,
            Random,
            rx0,
            [tx1, tx2]
        );
        crate::spawn_join!(source, selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(10, c1 + c2);
    }
}
