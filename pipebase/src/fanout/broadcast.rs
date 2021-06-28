use crate::{ConfigInto, FromConfig, FromPath, Select};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BroadcastConfig {}

impl FromPath for BroadcastConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(BroadcastConfig {})
    }
}

#[async_trait]
impl ConfigInto<Broadcast> for BroadcastConfig {}

pub struct Broadcast {}

#[async_trait]
impl FromConfig<BroadcastConfig> for Broadcast {
    async fn from_config(_config: &BroadcastConfig) -> anyhow::Result<Self> {
        Ok(Broadcast {})
    }
}

impl<T> Select<T, BroadcastConfig> for Broadcast {
    fn select(&mut self, _t: &T, candidates: &[&usize]) -> Vec<usize> {
        let mut all: Vec<usize> = Vec::new();
        for i in 0..candidates.len() {
            all.push(candidates[i].to_owned())
        }
        all
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
                    println!("id: {}, {}th tick, ticks: {}", id, tick, c);
                }
                None => return c,
            }
        }
    }
    #[tokio::test]
    async fn test_broadcast() {
        let (tx0, rx0) = channel!(u128, 1024);
        let (tx1, mut rx1) = channel!(u128, 1024);
        let (tx2, mut rx2) = channel!(u128, 1024);
        let mut source = poller!("timer", "resources/catalogs/timer.yml", TimerConfig, [tx0]);
        let mut selector = selector!("boradcast_select", BroadcastConfig, rx0, [tx1, tx2]);
        crate::spawn_join!(source, selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(20, c1 + c2);
    }
}
