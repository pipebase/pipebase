use super::Select;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RandomSelectorConfig {}

#[async_trait]
impl FromPath for RandomSelectorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(RandomSelectorConfig {})
    }
}

#[async_trait]
impl ConfigInto<RandomSelector> for RandomSelectorConfig {}

/// Select candidates with random number
pub struct RandomSelector {}

#[async_trait]
impl FromConfig<RandomSelectorConfig> for RandomSelector {
    async fn from_config(_config: RandomSelectorConfig) -> anyhow::Result<Self> {
        Ok(RandomSelector {})
    }
}

/// # Parameters
/// * T: input
#[async_trait]
impl<T> Select<T, RandomSelectorConfig> for RandomSelector
where
    T: Sync,
{
    /// `candidates`: index of downstreams
    async fn select(&mut self, _t: &T, candidates: &[&usize]) -> anyhow::Result<Vec<usize>> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..candidates.len());
        Ok(vec![candidates[i].to_owned()])
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
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
        let source = poller!("timer");
        let selector = selector!("random_select");
        join_pipes!([
            run_pipe!(source, TimerConfig, "resources/catalogs/timer.yml", [tx0]),
            run_pipe!(selector, RandomSelectorConfig, [tx1, tx2], rx0)
        ]);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(10, c1 + c2);
    }
}
