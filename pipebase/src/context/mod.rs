mod print;
mod runtime;

pub use print::*;
pub use runtime::*;

use async_trait::async_trait;

use crate::common::{Context, FromConfig};

#[async_trait]
pub trait StoreContext<C>: FromConfig<C> {
    fn store_context(&mut self, pipe_name: String, context: std::sync::Arc<Context>);

    fn load_context(&self, pipe_name: &str) -> Option<&std::sync::Arc<Context>>;

    async fn run(&mut self) -> anyhow::Result<()>;
}
