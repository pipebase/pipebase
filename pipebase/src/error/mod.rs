mod print;
mod runtime;

pub use print::*;
pub use runtime::*;

use crate::common::{FromConfig, PipeError};
use async_trait::async_trait;

#[async_trait]
pub trait HandleError<C>: FromConfig<C> {
    async fn handle_error(&mut self, pipe_error: PipeError) -> anyhow::Result<()>;
}
