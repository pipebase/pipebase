mod file;
mod runtime;

pub use file::*;
pub use runtime::*;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::FromConfig;

#[async_trait]
pub trait Listen<T, C>: Send + Sync + FromConfig<C>
where
    T: Send + 'static,
{
    async fn run(&mut self) -> anyhow::Result<()>;
    fn set_sender(&mut self, sender: Sender<T>);
}
