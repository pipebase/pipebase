mod bag;
mod runtime;
mod set;
mod text;
pub use bag::*;
pub use runtime::*;
pub use set::*;
pub use text::*;

use crate::common::FromConfig;

use async_trait::async_trait;
use tokio::time::Interval;

#[async_trait]
pub trait Collect<T, U, C>: Send + FromConfig<C> {
    async fn collect(&mut self, t: T) -> anyhow::Result<()>;
    async fn flush(&mut self) -> anyhow::Result<Option<U>>;
    fn get_flush_interval(&self) -> Interval;
}
