use crate::{ConfigInto, FromConfig, FromPath, Select};
use async_trait::async_trait;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RandomConfig {}

impl FromPath for RandomConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(RandomConfig {})
    }
}

#[async_trait]
impl ConfigInto<Random> for RandomConfig {}

pub struct Random {}

#[async_trait]
impl FromConfig<RandomConfig> for Random {
    async fn from_config(_config: &RandomConfig) -> anyhow::Result<Self> {
        Ok(Random {})
    }
}

impl<T> Select<T, RandomConfig> for Random {
    fn select(&mut self, _t: &T, candidates: &[&usize]) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..candidates.len());
        vec![candidates[i].to_owned()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel, poller, selector, Pipe};
    use crate::{Poller, Selector, TimerConfig};
    use tokio::sync::mpsc::Receiver;

    async fn count_tick(rx: &mut Receiver<u128>, id: usize) -> usize {
        let mut c: usize = 0;
        loop {
            match rx.recv().await {
                Some(tick) => {
                    c += 1;
                    println!("id: {}, {}th tick, local #ticks: {}", id, tick, c);
                }
                None => return c,
            }
        }
    }

    #[tokio::test]
    async fn test_random_select() {
        let (tx0, rx0) = channel!(u128, 1024);
        let (tx1, mut rx1) = channel!(u128, 1024);
        let (tx2, mut rx2) = channel!(u128, 1024);
        let mut source = poller!("timer", "resources/catalogs/timer.yml", TimerConfig, [tx0]);
        let mut selector = selector!("random_select", RandomConfig, rx0, [tx1, tx2]);
        crate::spawn_join!(source, selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(10, c1 + c2);
    }
}
