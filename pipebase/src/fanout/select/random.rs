use crate::{ConfigInto, FromConfig, FromFile, Select};
use async_trait::async_trait;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RandomConfig {
    pub n: usize,
}

impl FromFile for RandomConfig {}

#[async_trait]
impl ConfigInto<Random> for RandomConfig {}

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

impl Select<RandomConfig> for Random {
    fn select(&mut self) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.n);
        vec![i]
    }

    fn get_range(&mut self) -> usize {
        self.n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel, poller, selector, Pipe};
    use crate::{Poller, Selector, TimePollerConfig};
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
        let (tx0, rx0) = channel!((), 1024);
        let (tx1, mut rx1) = channel!((), 1024);
        let (tx2, mut rx2) = channel!((), 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimePollerConfig,
            [tx0]
        );
        let mut selector = selector!(
            "random_select",
            "resources/catalogs/random_selector.yml",
            RandomConfig,
            rx0,
            [tx1, tx2]
        );
        crate::spawn_join!(source, selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(10, c1 + c2);
    }
}
