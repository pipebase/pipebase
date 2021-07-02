mod print;

pub use print::*;
use tokio::sync::mpsc::Sender;

use async_trait::async_trait;
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::context::State;
use crate::Pipe;
use crate::Result;
use crate::{
    inc_success_run, inc_total_run, set_state, ConfigInto, Context, FromConfig, HasContext,
};

#[async_trait]
pub trait Export<T, C>: Send + Sync + FromConfig<C>
where
    T: Send + Sync + 'static,
{
    async fn export(&mut self, t: &T) -> anyhow::Result<()>;
}

pub struct Exporter<'a> {
    name: &'a str,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, E, C> Pipe<T, (), E, C> for Exporter<'a>
where
    T: Send + Sync + 'static,
    E: Export<T, C> + 'static,
    C: ConfigInto<E> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<()>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(txs.is_empty());
        let mut exporter = config.config_into().await.unwrap();
        let rx = rx.as_mut().unwrap();
        log::info!("exporter {} run ...", self.name);
        loop {
            inc_total_run(&self.context).await;
            set_state(&self.context, State::Receive).await;
            let t = match rx.recv().await {
                Some(t) => t,
                None => {
                    inc_success_run(&self.context).await;
                    break;
                }
            };
            set_state(&self.context, State::Export).await;
            match exporter.export(&t).await {
                Ok(_) => {
                    inc_success_run(&self.context).await;
                }
                Err(err) => {
                    error!("exporter error {}", err);
                    break;
                }
            }
        }
        log::info!("exporter {} exit ...", self.name);
        set_state(&self.context, State::Done).await;
        Ok(())
    }
}

impl<'a> HasContext for Exporter<'a> {
    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

impl<'a> Exporter<'a> {
    pub fn new(name: &'a str) -> Self {
        Exporter {
            name: name,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! exporter {
    (
        $name:expr
    ) => {{
        Exporter::new($name)
    }};
}
