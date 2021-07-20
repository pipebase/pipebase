mod print;
mod runtime;

pub use print::*;
pub use runtime::*;

use async_trait::async_trait;

use crate::common::FromConfig;

#[async_trait]
pub trait Export<T, C>: Send + Sync + FromConfig<C> {
    async fn export(&mut self, t: T) -> anyhow::Result<()>;
}
