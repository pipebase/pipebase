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
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: &T) -> std::result::Result<U, Box<dyn std::error::Error>>;
}

pub struct Mapper<'a, T, U, M: Map<T, U, C>, C: ConfigInto<M>> {
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub txs: Vec<Arc<Sender<U>>>,
    pub config: C,
    pub mapper: PhantomData<M>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<
        'a,
        T: Send + Sync,
        U: Clone + Debug + Send + 'static,
        M: Map<T, U, C>,
        C: ConfigInto<M> + Send + Sync,
    > Pipe<U> for Mapper<'a, T, U, M, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut mapper = self.config.config_into().await.unwrap();
        loop {
            Self::inc_total_run(self.context.clone()).await;
            Self::set_state(self.context.clone(), State::Receive).await;
            // if all receiver dropped, sender drop as well
            match self.txs.is_empty() {
                true => {
                    Self::inc_success_run(self.context.clone()).await;
                    break;
                }
                false => (),
            }
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    Self::inc_success_run(self.context.clone()).await;
                    break;
                }
            };
            Self::set_state(self.context.clone(), State::Process).await;
            let u = match mapper.map(&t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    break;
                }
            };
            Self::set_state(self.context.clone(), State::Send).await;
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let u_clone: U = u.to_owned();
                jhs.push(Self::spawn_send(tx, u_clone));
            }
            let dropped_receiver_idxs = Self::wait_join_handles(jhs).await;
            self.txs = Self::filter_sender_by_dropped_receiver_idx(
                self.txs.to_owned(),
                dropped_receiver_idxs,
            );
            Self::inc_success_run(self.context.clone()).await;
        }
        Self::set_state(self.context.clone(), State::Done).await;
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<U>) {
        self.txs.push(Arc::new(tx));
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
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Mapper {
                name: $name,
                rx: $rx,
                txs: vec![],
                config: config,
                mapper: std::marker::PhantomData,
                context: Default::default()
            };
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
        .await
    };
}
