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
        txs: Vec<Sender<U>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()>;
}

pub trait HasContext {
    fn get_context(&self) -> Arc<RwLock<Context>>;
}

#[macro_export]
macro_rules! run_pipe {
    {
        $pipe:ident, $config:ty, [$( $tx:expr ), *]
    } => {
        run_pipe!($pipe, $config, "", [$( $tx ), *], { None })
    };
    {
        $pipe:ident, $config:ty, [$( $tx:expr ), *], $rx:ident
    } => {
        run_pipe!($pipe, $config, "", [$( $tx ), *], { Some($rx) })
    };
    {
        $pipe:ident, $config:ty, $path:expr, [$( $tx:expr ), *]
    } => {
        run_pipe!($pipe, $config, $path, [$( $tx ), *], { None })
    };
    {
        $pipe:ident, $config:ty, $path:expr, [$( $tx:expr ), *], $rx:ident
    } => {
        run_pipe!($pipe, $config, $path, [$( $tx ), *], { Some($rx) })
    };
    {
        $pipe:ident, $config:ty, $path:expr, [$( $tx:expr ), *], $rx:expr
    } => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut txs = vec![];
            $(
                txs.push($tx);
            )*
            tokio::spawn(async move {
                match $pipe.run(config, txs, $rx).await {
                    Ok(context) => Ok(context),
                    Err(err) => {
                        log::error!("pipe exit with error {:#?}", err);
                        Err(err)
                    }
                }
            })
        }
    };
}

#[macro_export]
macro_rules! join_pipes {
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
