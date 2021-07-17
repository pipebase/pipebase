mod aggregate;
mod echo;
mod field;
mod file;
mod filter;
mod project;
mod runtime;
mod split;

pub use aggregate::*;
pub use echo::*;
pub use field::*;
pub use file::*;
pub use filter::*;
pub use project::*;
pub use runtime::*;
pub use split::*;

use async_trait::async_trait;

use crate::FromConfig;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: T) -> anyhow::Result<U>;
}
