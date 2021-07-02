use crate::{ConfigInto, Context, FromConfig, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

#[async_trait]
pub trait Pipe<T, U, R, C>: HasContext
where
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
    {
        $pipe:expr, $config:ty, $path:expr, $rx:expr, [$( $tx:expr ), *]
    } => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut txs = vec![];
            $(
                txs.push($tx);
            )*
            run_pipe!($pipe, config, $rx, txs)
        }
    }
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
    };
    (
        [$( $run_pipe:expr ), *]
    ) => {
        let _ = tokio::join!($(
            $run_pipe
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
