mod bootstrap;
mod context;
mod error;
mod fanout;
mod process;
mod sink;
mod source;
mod utils;

pub use bootstrap::*;
pub use context::*;
pub use fanout::*;
pub use pipederive::*;
pub use process::*;
pub use sink::*;
pub use source::*;
pub(crate) use utils::*;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

use error::Result;

pub trait FromPath: Sized + DeserializeOwned {
    fn from_path<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader::<std::fs::File, Self>(file)?;
        Ok(config)
    }
}

#[async_trait]
pub trait FromConfig<T>: Sized {
    async fn from_config(config: &T) -> anyhow::Result<Self>;
}

#[async_trait]
pub trait ConfigInto<T: FromConfig<Self>>: Sized {
    async fn config_into(&self) -> anyhow::Result<T> {
        T::from_config(self).await
    }
}

#[async_trait]
pub trait Pipe<T, U, R, C>
where
    U: Send + 'static,
    R: FromConfig<C>,
    C: ConfigInto<R>,
{
    async fn run(
        &mut self,
        config: C,
        mut rx: Option<Receiver<T>>,
        txs: Vec<Sender<U>>,
    ) -> Result<()>;
}

pub trait HasContext {
    fn get_context(&self) -> Arc<RwLock<Context>>;
}

#[macro_export]
macro_rules! run_pipe {
    (
        $pipe:expr, $config:expr, $rx:expr, $txs:expr
    ) => {
        tokio::spawn(async move {
            match $pipe.run($config, $rx, $txs).await {
                Ok(context) => Ok(context),
                Err(err) => {
                    log::error!("pipe exit with error {:#?}", err);
                    Err(err)
                }
            }
        })
    };
}

#[macro_export]
macro_rules! run_pipes {
    (
        [$( ($pipe:expr, $config:ty, $path:expr, $rx:expr, [$( $tx:expr ), *]) ), *]
    ) => {
        let _ = tokio::join!($(
            {
                let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
                let mut txs = vec![];
                $(
                    txs.push($tx);
                )*
                run_pipe!($pipe, config, $rx, txs)
            }
        ),*);
    }
}

#[macro_export]
macro_rules! channel {
    (
        $ty:ty, $size:expr
    ) => {{
        use tokio::sync::mpsc::channel;
        channel::<$ty>($size)
    }};
    (
        $path:path, $size:expr
    ) => {{
        use tokio::sync::mpsc::channel;
        channel::<$path>($size)
    }};
    (
        $expr:expr, $size:expr
    ) => {{
        use tokio::sync::mpsc::channel;
        channel::<$expr>($size)
    }};
}
