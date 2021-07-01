mod print;

pub use print::*;

use std::marker::PhantomData;

use async_trait::async_trait;
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::context::State;
use crate::Pipe;
use crate::Result;
use crate::{context::Context, ConfigInto, FromConfig};

#[async_trait]
pub trait Export<T, C>: Send + Sync + FromConfig<C>
where
    T: Send + Sync + 'static,
{
    async fn export(&mut self, t: &T) -> anyhow::Result<()>;
}

pub struct Exporter<'a, T, E, C>
where
    T: Send + Sync + 'static,
    E: Export<T, C>,
    C: ConfigInto<E>,
{
    name: &'a str,
    config: C,
    rx: Receiver<T>,
    exporter: PhantomData<E>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, E, C> Pipe<T, (), E, C> for Exporter<'a, T, E, C>
where
    T: Send + Sync + 'static,
    E: Export<T, C> + 'static,
    C: ConfigInto<E> + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        let mut exporter = self.config.config_into().await.unwrap();
        log::info!("exporter {} run ...", self.name);
        loop {
            Self::inc_total_run(&self.context).await;
            Self::set_state(&self.context, State::Receive).await;
            let t = match self.rx.recv().await {
                Some(t) => t,
                None => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
            };
            Self::set_state(&self.context, State::Export).await;
            match exporter.export(&t).await {
                Ok(_) => {
                    Self::inc_success_run(&self.context).await;
                }
                Err(err) => {
                    error!("exporter error {}", err);
                    break;
                }
            }
        }
        log::info!("exporter {} exit ...", self.name);
        Self::set_state(&self.context, State::Done).await;
        Ok(())
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.to_owned()
    }
}

impl<'a, T, E, C> Exporter<'a, T, E, C>
where
    T: Send + Sync + 'static,
    E: Export<T, C>,
    C: ConfigInto<E>,
{
    pub fn new(name: &'a str, config: C, rx: Receiver<T>) -> Self {
        Exporter {
            name: name,
            config: config,
            rx: rx,
            exporter: std::marker::PhantomData,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! exporter {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr
    ) => {{
        let config =
            <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
        let pipe = Exporter::new($name, config, $rx);
        pipe
    }};
    (
        $name:expr, $config:ty, $rx:expr
    ) => {
        exporter!($name, "", $config, $rx)
    };
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        exporter!($name, $path, $config, $rx)
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        exporter!($name, "", $config, $rx)
    };
}
