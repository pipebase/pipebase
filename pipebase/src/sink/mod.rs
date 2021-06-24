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
    async fn export(
        &mut self,
        t: &T,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Exporter<'a, T, E, C>
where
    T: Send + Sync + 'static,
    E: Export<T, C>,
    C: ConfigInto<E>,
{
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub config: C,
    pub exporter: PhantomData<E>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, E, C> Pipe<()> for Exporter<'a, T, E, C>
where
    T: Send + Sync + 'static,
    E: Export<T, C> + 'static,
    C: ConfigInto<E> + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        let mut exporter = self.config.config_into().await.unwrap();

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
                    error!("export error {}", err);
                    break;
                }
            }
        }
        Self::set_state(&self.context, State::Done).await;

        Ok(())
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.to_owned()
    }
}

#[macro_export]
macro_rules! exporter {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr
    ) => {{
        let config =
            <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
        let pipe = Exporter {
            name: $name,
            rx: $rx,
            config: config,
            exporter: std::marker::PhantomData,
            context: Default::default(),
        };
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
