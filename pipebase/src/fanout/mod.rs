mod broadcast;
mod hash;
mod random;
mod roundrobin;

pub use broadcast::*;
pub use hash::*;
pub use random::*;
pub use roundrobin::*;

use crate::{ConfigInto, FromConfig, Pipe};
use async_trait::async_trait;
use std::marker::PhantomData;

use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::{select_range_error, Result};

pub trait Select<T, C>: Send + Sync + FromConfig<C> {
    fn select(&mut self, t: &T) -> Vec<usize>;
    fn get_range(&mut self) -> usize;
}

pub struct Selector<'a, T, S: Select<T, C>, C: ConfigInto<S>> {
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub txs: Vec<Arc<Sender<T>>>,
    pub config: C,
    pub selector: PhantomData<S>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Clone + Send + 'static, S: Select<T, C>, C: ConfigInto<S> + Send + Sync> Pipe<T>
    for Selector<'a, T, S, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut selector = self.config.config_into().await.unwrap();
        let selector_range = selector.get_range();
        let sender_range = self.txs.len();
        match selector_range == sender_range {
            false => {
                return Err(select_range_error(&format!(
                    "selector/sender range not equal {} != {}",
                    selector_range, sender_range
                )))
            }
            _ => (),
        }
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
            Self::set_state(&self.context, State::Send).await;
            let mut jhs = vec![];
            for i in selector.select(&t) {
                let tx = self.txs.get(i).unwrap().to_owned();
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
macro_rules! selector {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Selector {
                name: $name,
                rx: $rx,
                txs: vec![],
                config: config,
                selector: std::marker::PhantomData,
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
        selector!($name, "", $config, $rx, [$( $tx ), *])
    };
}
