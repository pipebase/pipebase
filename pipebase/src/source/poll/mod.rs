mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    async fn poll(&mut self) -> anyhow::Result<Option<T>>;
}

pub struct Poller<'a, T, P, C>
where
    T: Clone + Send + 'static,
    P: Poll<T, C>,
    C: ConfigInto<P> + Send + Sync,
{
    name: &'a str,
    txs: HashMap<usize, Arc<Sender<T>>>,
    config: C,
    poller: PhantomData<P>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, P, C> Pipe<T> for Poller<'a, T, P, C>
where
    T: Clone + Send + 'static,
    P: Poll<T, C>,
    C: ConfigInto<P> + Send + Sync,
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
            let mut jhs = HashMap::new();
            for (idx, tx) in &self.txs {
                let t_clone = t.to_owned();
                jhs.insert(idx.to_owned(), Self::spawn_send(tx.to_owned(), t_clone));
            }
            let drop_sender_indices = Self::wait_join_handles(jhs).await;
            Self::filter_senders_by_indices(&mut self.txs, drop_sender_indices);
            Self::inc_success_run(&self.context).await;
        }
        Self::set_state(&self.context, State::Done).await;
        info!("source {} exit ...", self.name);
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<T>) {
        let idx = self.txs.len();
        self.txs.insert(idx, Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

impl<'a, T, P, C> Poller<'a, T, P, C>
where
    T: Clone + Send + 'static,
    P: Poll<T, C>,
    C: ConfigInto<P> + Send + Sync,
{
    pub fn new(name: &'a str, config: C) -> Self {
        Poller {
            name: name,
            txs: HashMap::new(),
            config: config,
            poller: std::marker::PhantomData,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! poller {
    (
        $name:expr, $path:expr, $config:ty, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Poller::new($name, config);
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
