mod file;
mod iterator;
mod runtime;

pub use file::*;
pub use iterator::*;
pub use runtime::*;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::FromConfig;

#[async_trait]
pub trait Stream<T, U, C>: Send + Sync + FromConfig<C>
where
    U: Send + 'static,
{
    async fn stream(&mut self, t: T) -> anyhow::Result<()>;
    fn set_sender(&mut self, sender: Sender<U>);
}
