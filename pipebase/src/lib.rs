mod context;
mod error;
mod fanout;
mod process;
mod source;

use context::Context;
use context::State;
pub use fanout::*;
pub use pipederive::*;
pub use process::*;
use serde::de::DeserializeOwned;
pub use source::*;

use async_trait::async_trait;
use log::error;
use std::sync::Arc;
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
pub trait Pipe<T: Send + 'static> {
    async fn run(&mut self) -> Result<Arc<RwLock<Context>>>;

    fn add_sender(&mut self, tx: Sender<T>);

    fn spawn_send(tx: Arc<Sender<T>>, t: T) -> JoinHandle<()> {
        tokio::spawn(async move {
            match tx.send(t).await {
                Ok(()) => (),
                Err(err) => {
                    error!("selector send error {}", err.to_string());
                }
            }
        })
    }

    async fn wait_join_handles(join_handles: Vec<JoinHandle<()>>) {
        for jh in join_handles {
            match jh.await {
                Ok(()) => (),
                Err(err) => {
                    error!("join error in pipe err: {:#?}", err)
                }
            }
        }
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

            tokio::join!($(
                tokio::spawn(async move {
                    match $pipe.run().await {
                        Ok(context) => Ok(context),
                        Err(err) => {
                            log::error!("pipe exit with error {:#?}", err);
                            Err(err)
                        }
                    }
                })
            ),*)

    };
}

#[macro_export]
macro_rules! channel {
    (
        $ty:ty, $size:expr
    ) => {
        channel::<$ty>($size)
    };
    (
        $path:path, $size:expr
    ) => {
        channel::<$path>($size)
    };
    (
        $expr:expr, $size:expr
    ) => {
        channel::<$expr>($size)
    };
}
