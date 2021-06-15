mod print;

pub use print::*;

use std::marker::PhantomData;

use async_trait::async_trait;
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::{Mutex, RwLock};

use crate::context::State;
use crate::Pipe;
use crate::{context::Context, ConfigInto, FromConfig};
use crate::{error::join_error, Result};

#[async_trait]
pub trait Export<T: Send + Sync + 'static, C>: Send + Sync + FromConfig<C> {
    async fn export(
        &mut self,
        t: &T,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Sink<'a, T: Send + Sync + 'static, E: Export<T, C>, C: ConfigInto<E>> {
    pub name: &'a str,
    pub rx: Arc<Mutex<Receiver<T>>>,
    pub config: C,
    pub exporter: PhantomData<E>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Send + Sync + 'static, E: Export<T, C> + 'static, C: ConfigInto<E> + Send + Sync>
    Pipe<()> for Sink<'a, T, E, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut exporter = self.config.config_into().await.unwrap();
        let (tx_e, mut rx_e) = channel::<T>(1024);
        let rx = self.rx.to_owned();
        let event_loop = tokio::spawn(async move {
            let mut rx = rx.lock().await;
            loop {
                let t = match rx.recv().await {
                    Some(t) => t,
                    None => break,
                };
                match tx_e.send(t).await {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("send exporter failed {}", err);
                        break;
                    }
                }
            }
        });
        let context = self.context.to_owned();
        let export_loop = tokio::spawn(async move {
            loop {
                Self::inc_total_run(context.to_owned()).await;
                Self::set_state(context.to_owned(), State::Receive).await;
                let t = match rx_e.recv().await {
                    Some(t) => t,
                    None => {
                        Self::inc_success_run(context.to_owned()).await;
                        break;
                    }
                };
                Self::set_state(context.to_owned(), State::Export).await;
                match exporter.export(&t).await {
                    Ok(_) => {
                        Self::inc_success_run(context.to_owned()).await;
                    }
                    Err(err) => {
                        error!("export error {}", err);
                        break;
                    }
                }
            }
            Self::set_state(context.to_owned(), State::Done).await;
        });
        let join_all = tokio::spawn(async move { tokio::join!(event_loop, export_loop) });
        match join_all.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.to_owned()
    }
}

#[macro_export]
macro_rules! sink {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr
    ) => {
        async move {
            let config = <$config>::from_file($path)
                .expect(&format!("invalid config file location {}", $path));
            let pipe = Sink {
                name: $name,
                rx: std::sync::Arc::new(tokio::sync::Mutex::new($rx)),
                config: config,
                exporter: std::marker::PhantomData,
                context: Default::default(),
            };
            pipe
        }
        .await
    };
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        sink!($name, $path, $config, $rx)
    };
}
