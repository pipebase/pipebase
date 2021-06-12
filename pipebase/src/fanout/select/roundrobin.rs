use crate::{ConfigInto, FromConfig, FromFile, Select};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RoundRobinConfig {
    pub n: usize,
}

impl FromFile for RoundRobinConfig {}

#[async_trait]
impl ConfigInto<RoundRobin> for RoundRobinConfig {}

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

impl Select<RoundRobinConfig> for RoundRobin {
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

#[cfg(test)]
mod tests {}
