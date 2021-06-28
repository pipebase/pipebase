use crate::{ConfigInto, FromConfig, FromPath, Select};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RoundRobinConfig {}

impl FromPath for RoundRobinConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(RoundRobinConfig {})
    }
}

#[async_trait]
impl ConfigInto<RoundRobin> for RoundRobinConfig {}

pub struct RoundRobin {
    pub i: usize,
}

#[async_trait]
impl FromConfig<RoundRobinConfig> for RoundRobin {
    async fn from_config(_config: &RoundRobinConfig) -> anyhow::Result<Self> {
        Ok(RoundRobin { i: 0 })
    }
}

impl<T> Select<T, RoundRobinConfig> for RoundRobin {
    fn select<'a>(&mut self, _t: &T, candidates: &'a [&'a usize]) -> &'a [&'a usize] {
        let i = self.i % candidates.len();
        self.i = i + 1;
        &candidates[i..i + 1]
    }
}

#[cfg(test)]
mod tests {}
