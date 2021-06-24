mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::marker::PhantomData;
use std::sync::Arc;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    async fn poll(
        &mut self,
    ) -> std::result::Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Poller<'a, T, P: Poll<T, C>, C: ConfigInto<P>> {
    pub name: &'a str,
    pub txs: Vec<Arc<Sender<T>>>,
    pub config: C,
    pub poller: PhantomData<P>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Clone + Send + 'static, P: Poll<T, C>, C: ConfigInto<P> + Send + Sync> Pipe<T>
    for Poller<'a, T, P, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut poller = self.config.config_into().await.unwrap();
        loop {
            Self::inc_total_run(&self.context).await;
            Self::set_state(&self.context, State::Poll).await;
            // if all receiver dropped, sender drop as well
            match self.txs.is_empty() {
                true => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let t = poller.poll().await;
            let t = match t {
                Ok(t) => t,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    break;
                }
            };
            let t = match t {
                Some(t) => t,
                None => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
            };
            Self::set_state(&self.context, State::Send).await;
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let t_clone = t.to_owned();
                jhs.push(Self::spawn_send(tx, t_clone));
            }
            let dropped_receiver_idxs = Self::wait_join_handles(jhs).await;
            self.txs = Self::filter_sender_by_dropped_receiver_idx(
                self.txs.to_owned(),
                dropped_receiver_idxs,
            );
            Self::inc_success_run(&self.context).await;
        }
        Self::set_state(&self.context, State::Done).await;
        info!("source {} exit ...", self.name);
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! poller {
    (
        $name:expr, $path:expr, $config:ty, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Poller {
                name: $name,
                txs: vec![],
                config: config,
                poller: std::marker::PhantomData,
                context: Default::default()
            };
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
    };
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        poller!($name, $path, $config, [$( $tx ), *])
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        poller!($name, "", $config, [$( $tx ), *])
    };
    (
        $name:expr, $config:ty, [$( $tx:expr ), *]
    ) => {
        poller!($name, "", $config, [$( $tx ), *])
    };
}
