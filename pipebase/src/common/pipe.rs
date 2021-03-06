use super::{ConfigInto, FromConfig, HasContext, Result, SubscribeError};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::error;

pub struct PipeChannels<T, U> {
    rx: Option<Receiver<T>>,
    txs: Vec<Sender<U>>,
}

impl<T, U> PipeChannels<T, U> {
    pub fn rx(mut self, rx: Receiver<T>) -> Self {
        self.rx = Some(rx);
        self
    }

    pub fn tx(mut self, tx: Sender<U>) -> Self {
        self.txs.push(tx);
        self
    }

    pub fn into_channels(self) -> (Option<Receiver<T>>, Vec<Sender<U>>) {
        (self.rx, self.txs)
    }
}

impl<T, U> Default for PipeChannels<T, U> {
    fn default() -> Self {
        PipeChannels {
            rx: None,
            txs: vec![],
        }
    }
}

#[async_trait]
pub trait Pipe<T, U, R, C>: HasContext + SubscribeError
where
    R: FromConfig<C>,
    C: ConfigInto<R>,
{
    async fn run(self, config: C, channels: PipeChannels<T, U>) -> Result<()>;
}

// Sender Operations
pub(crate) fn senders_as_map<U>(txs: Vec<Sender<U>>) -> HashMap<usize, Sender<U>> {
    txs.into_iter().enumerate().into_iter().collect()
}

pub(crate) fn spawn_send<U>(
    tx: Sender<U>,
    t: U,
) -> JoinHandle<core::result::Result<(), SendError<U>>>
where
    U: Send + 'static,
{
    tokio::spawn(async move {
        match tx.send(t).await {
            Ok(()) => Ok(()),
            Err(err) => {
                error!("selector send error {}", err.to_string());
                Err(err)
            }
        }
    })
}

pub(crate) async fn wait_join_handles<U>(
    join_handles: HashMap<usize, JoinHandle<core::result::Result<(), SendError<U>>>>,
) -> Vec<usize> {
    let mut drop_sender_indices = Vec::new();
    for (idx, jh) in join_handles {
        let result = match jh.await {
            Ok(res) => res,
            Err(err) => {
                error!("join error in pipe err: {:#?}", err);
                drop_sender_indices.push(idx);
                continue;
            }
        };
        match result {
            Ok(()) => (),
            Err(err) => {
                error!("send error {}", err);
                drop_sender_indices.push(idx);
            }
        }
    }
    drop_sender_indices
}

pub(crate) fn filter_senders_by_indices<U>(
    senders: &mut HashMap<usize, Sender<U>>,
    remove_indices: Vec<usize>,
) {
    for idx in remove_indices {
        senders.remove(&idx);
    }
}

pub(crate) fn replicate<U>(u: U, r: usize) -> Vec<U>
where
    U: Clone,
{
    let mut replicas: Vec<U> = Vec::new();
    for _ in 0..r - 1 {
        replicas.push(u.to_owned());
    }
    replicas.push(u);
    replicas
}

#[cfg(test)]
pub(crate) async fn populate_records<T, U>(tx: Sender<T>, records: U)
where
    U: IntoIterator<Item = T>,
{
    for record in records {
        let _ = tx.send(record).await;
    }
}

#[macro_export]
macro_rules! pipe_channels {
    {
        $rx:ident
    } => {
        {
            PipeChannels::default().rx($rx)
        }
    };
    {
        [$( $tx:expr ), *]
    } => {
        {
            PipeChannels::default()$(
                .tx($tx)
            )*
        }
    };
    {
        $rx:ident, [$( $tx:expr ), *]
    } => {
        {
            PipeChannels::default().rx($rx)$(
                .tx($tx)
            )*
        }
    };
}

#[macro_export]
macro_rules! run_pipe {
    {
        $pipe:ident, $config:ident, $channels:ident
    } => {
        {
            tokio::spawn(async move {
                match $pipe.run($config, $channels).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        tracing::error!("pipe exit with error {:#?}", err);
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
