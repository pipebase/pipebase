mod runtime;
mod timer;

pub use runtime::*;
pub use timer::*;

use crate::common::FromConfig;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::Interval;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    // return None if it's the end
    async fn poll(&mut self) -> anyhow::Result<Option<T>>;

    fn get_initial_delay(&self) -> Duration;

    fn get_interval(&self) -> Interval;
}
