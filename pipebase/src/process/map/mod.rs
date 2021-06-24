mod echo;
mod field;
mod filter;
mod project;

pub use echo::*;
pub use field::*;
pub use filter::*;
pub use project::*;

use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: &T) -> std::result::Result<U, Box<dyn std::error::Error>>;
}

pub struct Mapper<'a, T, U, M, C>
where
    T: Send + Sync,
    U: Clone + Debug + Send + 'static,
    M: Map<T, U, C>,
    C: ConfigInto<M> + Send + Sync,
{
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub txs: HashMap<usize, Arc<Sender<U>>>,
    pub config: C,
    pub mapper: PhantomData<M>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, U, M, C> Pipe<U> for Mapper<'a, T, U, M, C>
where
    T: Send + Sync,
    U: Clone + Debug + Send + 'static,
    M: Map<T, U, C>,
    C: ConfigInto<M> + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        let mut mapper = self.config.config_into().await.unwrap();
        loop {
            Self::inc_total_run(&self.context).await;
            Self::set_state(&self.context, State::Receive).await;
            // if all receiver dropped, sender drop as well
            match self.txs.is_empty() {
                true => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
            };
            Self::set_state(&self.context, State::Process).await;
            let u = match mapper.map(&t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    break;
                }
            };
            Self::set_state(&self.context, State::Send).await;
            let mut jhs = HashMap::new();
            for (idx, tx) in &self.txs {
                let u_clone: U = u.to_owned();
                jhs.insert(idx.to_owned(), Self::spawn_send(tx.to_owned(), u_clone));
            }
            let drop_sender_indices = Self::wait_join_handles(jhs).await;
            Self::filter_senders_by_indices(&mut self.txs, drop_sender_indices);
            Self::inc_success_run(&self.context).await;
        }
        Self::set_state(&self.context, State::Done).await;
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<U>) {
        let idx = self.txs.len();
        self.txs.insert(idx, Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! mapper {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Mapper {
                name: $name,
                rx: $rx,
                txs: std::collections::HashMap::new(),
                config: config,
                mapper: std::marker::PhantomData,
                context: Default::default()
            };
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        mapper!($name, "", $config, $rx, [$( $tx ), *])
    };
}
