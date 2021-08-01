mod hash;
mod random;
mod roundrobin;
mod runtime;

pub use hash::*;
pub use random::*;
pub use roundrobin::*;
pub use runtime::*;

use crate::common::FromConfig;
use async_trait::async_trait;

#[async_trait]
pub trait Select<T, C>: Send + Sync + FromConfig<C> {
    async fn select(&mut self, t: &T, candidates: &[&usize]) -> anyhow::Result<Vec<usize>>;
}
