mod bootstrap;
mod context;
mod error;
mod fanout;
mod process;
mod sink;
mod source;

pub use bootstrap::*;
pub use fanout::*;
pub use pipederive::*;
pub use process::*;
pub use sink::*;
pub use source::*;

use context::Context;
use context::State;

use async_trait::async_trait;
use log::error;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use error::Result;

pub trait FromFile: Sized + DeserializeOwned {
    fn from_file(path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader::<std::fs::File, Self>(file)?;
        Ok(config)
    }
}

#[async_trait]
pub trait FromConfig<T>: Sized {
    async fn from_config(config: &T) -> std::result::Result<Self, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait ConfigInto<T: FromConfig<Self>>: Sized {
    async fn config_into(&self) -> std::result::Result<T, Box<dyn std::error::Error>> {
        T::from_config(self).await
    }
}

#[async_trait]
pub trait Pipe<T: Send + 'static> {
    async fn run(&mut self) -> Result<()>;

    fn add_sender(&mut self, _tx: Sender<T>) {}

    fn spawn_send(tx: Arc<Sender<T>>, t: T) -> JoinHandle<core::result::Result<(), SendError<T>>> {
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

    async fn wait_join_handles(
        join_handles: Vec<JoinHandle<core::result::Result<(), SendError<T>>>>,
    ) -> HashSet<usize> {
        let mut i: usize = 0;
        let mut dropped_receiver_idxs = HashSet::new();
        for jh in join_handles {
            let result = match jh.await {
                Ok(res) => res,
                Err(err) => {
                    error!("join error in pipe err: {:#?}", err);
                    dropped_receiver_idxs.insert(i);
                    i += 1;
                    continue;
                }
            };
            match result {
                Ok(()) => (),
                Err(err) => {
                    error!("send error {}", err);
                    dropped_receiver_idxs.insert(i);
                }
            }
            i += 1;
        }
        dropped_receiver_idxs
    }

    fn filter_sender_by_dropped_receiver_idx(
        senders: Vec<Arc<Sender<T>>>,
        dropped_receiver_idxs: HashSet<usize>,
    ) -> Vec<Arc<Sender<T>>> {
        let mut healthy_senders: Vec<Arc<Sender<T>>> = vec![];
        let mut i: usize = 0;
        let len = senders.len();
        while i < len {
            if !dropped_receiver_idxs.contains(&i) {
                healthy_senders.push(senders.get(i).unwrap().to_owned());
            }
            i += 1;
        }
        healthy_senders
    }

    async fn set_state(context: Arc<RwLock<Context>>, state: State) {
        let mut ctx = context.write().await;
        ctx.set_state(state)
    }

    async fn inc_total_run(context: Arc<RwLock<Context>>) {
        let mut ctx = context.write().await;
        ctx.inc_total_run()
    }

    async fn inc_success_run(context: Arc<RwLock<Context>>) {
        let mut ctx = context.write().await;
        ctx.inc_success_run()
    }

    fn get_context(&self) -> Arc<RwLock<Context>>;
}

#[macro_export]
macro_rules! spawn_join {
    (
        $( $pipe:expr ), *
    ) => {

            let _ = tokio::join!($(
                tokio::spawn(async move {
                    match $pipe.run().await {
                        Ok(context) => Ok(context),
                        Err(err) => {
                            log::error!("pipe exit with error {:#?}", err);
                            Err(err)
                        }
                    }
                })
            ),*);
    };
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
