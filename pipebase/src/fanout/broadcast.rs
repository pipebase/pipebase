use crate::{ConfigInto, FromConfig, FromFile, Select};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BroadcastConfig {
    pub n: usize,
}

impl FromFile for BroadcastConfig {}

#[async_trait]
impl ConfigInto<Broadcast> for BroadcastConfig {}

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

impl<T> Select<T, BroadcastConfig> for Broadcast {
    fn select(&mut self, _t: &T) -> Vec<usize> {
        (0..self.n).collect()
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
    async fn test_broadcast() {
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
            "boradcast_select",
            "resources/catalogs/broadcast_selector.yml",
            BroadcastConfig,
            rx0,
            [tx1, tx2]
        );
        crate::spawn_join!(source, selector);
        let c1 = count_tick(&mut rx1, 0).await;
        let c2 = count_tick(&mut rx2, 1).await;
        assert_eq!(20, c1 + c2);
    }
}
