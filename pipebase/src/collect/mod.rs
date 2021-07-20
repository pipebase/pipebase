mod bag;
mod runtime;
mod set;
pub use bag::*;
pub use runtime::*;
pub use set::*;

use std::iter::FromIterator;

use crate::common::FromConfig;

use async_trait::async_trait;
use tokio::time::Interval;

#[async_trait]
pub trait Collect<T, U, C>: Send + FromConfig<C>
where
    U: FromIterator<T> + Clone,
{
    async fn collect(&mut self, t: T);
    async fn flush(&mut self) -> U;
    fn get_flush_interval(&self) -> Interval;
}
