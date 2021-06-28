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
use std::collections::HashMap;
use std::marker::PhantomData;

use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;

pub trait Select<T, C>: Send + Sync + FromConfig<C> {
    fn select<'a>(&mut self, t: &T, candidates: &'a [&'a usize]) -> &'a [&'a usize];
}

pub struct Selector<'a, T, S, C>
where
    T: Clone + Send + 'static,
    S: Select<T, C>,
    C: ConfigInto<S> + Send + Sync,
{
    name: &'a str,
    config: C,
    rx: Receiver<T>,
    txs: HashMap<usize, Arc<Sender<T>>>,
    selector: PhantomData<S>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, S, C> Pipe<T> for Selector<'a, T, S, C>
where
    T: Clone + Send + 'static,
    S: Select<T, C>,
    C: ConfigInto<S> + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        let mut selector = self.config.config_into().await.unwrap();
        log::info!("selector {} run ...", self.name);
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
            let candidates = self.txs.keys().collect::<Vec<&usize>>();
            let mut jhs = HashMap::new();
            for i in selector.select(&t, &candidates).to_owned() {
                let tx = self.txs.get(i).unwrap();
                let t_clone = t.to_owned();
                jhs.insert(i.to_owned(), Self::spawn_send(tx.to_owned(), t_clone));
            }
            let drop_sender_indices = Self::wait_join_handles(jhs).await;
            Self::filter_senders_by_indices(&mut self.txs, drop_sender_indices);
            Self::inc_success_run(&self.context).await;
        }
        log::info!("selector {} exit ...", self.name);
        Self::set_state(&self.context, State::Done).await;
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

impl<'a, T, S, C> Selector<'a, T, S, C>
where
    T: Clone + Send + 'static,
    S: Select<T, C>,
    C: ConfigInto<S> + Send + Sync,
{
    pub fn new(name: &'a str, config: C, rx: Receiver<T>) -> Self {
        Selector {
            name: name,
            config: config,
            rx: rx,
            txs: HashMap::new(),
            selector: std::marker::PhantomData,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! selector {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Selector::new($name, config, $rx);
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
