use super::Select;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RoundRobinSelectorConfig {}

#[async_trait]
impl FromPath for RoundRobinSelectorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(RoundRobinSelectorConfig {})
    }
}

#[async_trait]
impl ConfigInto<RoundRobinSelector> for RoundRobinSelectorConfig {}

/// Select candidates use round robin
pub struct RoundRobinSelector {
    pub i: usize,
}

#[async_trait]
impl FromConfig<RoundRobinSelectorConfig> for RoundRobinSelector {
    async fn from_config(_config: RoundRobinSelectorConfig) -> anyhow::Result<Self> {
        Ok(RoundRobinSelector { i: 0 })
    }
}

/// # Parameters
/// * T: input
#[async_trait]
impl<T> Select<T, RoundRobinSelectorConfig> for RoundRobinSelector
where
    T: Sync,
{
    /// `candidates`: index of downstreams
    async fn select(&mut self, _t: &T, candidates: &[&usize]) -> anyhow::Result<Vec<usize>> {
        let i = self.i % candidates.len();
        let selected = vec![candidates[i].clone()];
        self.i = i + 1;
        Ok(selected)
    }
}

#[cfg(test)]
mod tests {}
