mod add;
mod conversion;
mod echo;
mod field;
mod file;
mod filter;
mod project;
mod runtime;
mod sort;
mod split;

pub use add::*;
pub use conversion::*;
pub use echo::*;
pub use field::*;
pub use file::*;
pub use filter::*;
pub use project::*;
pub use runtime::*;
pub use sort::*;
pub use split::*;

use async_trait::async_trait;

use crate::common::FromConfig;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: T) -> anyhow::Result<U>;
}
