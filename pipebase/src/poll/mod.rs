mod runtime;
mod timer;

pub use runtime::*;
pub use timer::*;

use async_trait::async_trait;

use crate::common::FromConfig;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    async fn poll(&mut self) -> anyhow::Result<Option<T>>;
}
