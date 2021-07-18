use crate::{ConfigInto, FromConfig, FromPath, Select};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RoundRobinConfig {}

#[async_trait]
impl FromPath for RoundRobinConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(RoundRobinConfig {})
    }
}

#[async_trait]
impl ConfigInto<RoundRobin> for RoundRobinConfig {}

/// Select candidates use round robin
pub struct RoundRobin {
    pub i: usize,
}

#[async_trait]
impl FromConfig<RoundRobinConfig> for RoundRobin {
    async fn from_config(_config: RoundRobinConfig) -> anyhow::Result<Self> {
        Ok(RoundRobin { i: 0 })
    }
}

/// # Parameters
/// * T: input
impl<T> Select<T, RoundRobinConfig> for RoundRobin {
    /// `candidates`: index of downstreams
    fn select(&mut self, _t: &T, candidates: &[&usize]) -> Vec<usize> {
        let i = self.i % candidates.len();
        let selected = vec![candidates[i].clone()];
        self.i = i + 1;
        selected
    }
}

#[cfg(test)]
mod tests {}
