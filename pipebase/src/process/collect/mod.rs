mod bag;
mod set;
pub use bag::*;
pub use set::*;

use std::iter::FromIterator;
use std::marker::PhantomData;

use crate::{
    context::{Context, State},
    error::join_error,
    ConfigInto, FromConfig, Pipe, Result,
};

use async_trait::async_trait;
use std::sync::Arc;
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    },
    time::Interval,
};

#[async_trait]
pub trait Collect<T: Clone, U: FromIterator<T> + Clone, C>: Send + Sync + FromConfig<C> {
    async fn collect(&mut self, t: &T);
    async fn flush(&mut self) -> U;
    fn get_flush_interval(&self) -> Interval;
}

pub struct Collector<
    'a,
    T: Clone,
    U: FromIterator<T> + Clone,
    V: Collect<T, U, C>,
    C: ConfigInto<V>,
> {
    pub name: &'a str,
    pub rx: Arc<Mutex<Receiver<T>>>,
    pub txs: Vec<Arc<Sender<U>>>,
    pub config: C,
    pub collector: PhantomData<V>,
    pub collection: PhantomData<U>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<
        'a,
        T: Clone + Send + Sync + 'static,
        U: FromIterator<T> + Clone + Send + 'static,
        V: Collect<T, U, C> + 'static,
        C: ConfigInto<V> + Send + Sync,
    > Pipe<U> for Collector<'a, T, U, V, C>
{
    async fn run(&mut self) -> Result<()> {
        let collector: Arc<Mutex<V>> =
            Arc::new(Mutex::new(self.config.config_into().await.unwrap()));
        let rx = self.rx.to_owned();
        let collector_clone = collector.to_owned();
        let is_end = Arc::new(Mutex::new(false));
        let is_end_clone = is_end.to_owned();
        let join_event = tokio::spawn(async move {
            let mut rx = rx.lock().await;
            loop {
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => {
                        let mut is_end = is_end_clone.lock().await;
                        *is_end = true;
                        break;
                    }
                };
                let mut c = collector_clone.lock().await;
                (*c).collect(&t).await;
            }
        });
        let collector_clone = collector.to_owned();
        let mut txs = self.txs.to_owned();
        let is_end_clone = is_end.to_owned();
        let context = self.get_context();
        let join_flush = tokio::spawn(async move {
            let mut interval = {
                let c = collector_clone.lock().await;
                c.get_flush_interval()
            };
            loop {
                Self::set_state(&context, State::Receive).await;
                Self::inc_total_run(&context).await;
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        Self::inc_success_run(&context).await;
                        break;
                    }
                    false => (),
                }
                interval.tick().await;
                let mut c = collector_clone.lock().await;
                let data = c.flush().await;
                Self::set_state(&context, State::Send).await;
                let mut jhs = vec![];
                for tx in txs.as_slice() {
                    let tx_clone = tx.to_owned();
                    let data_clone = data.to_owned();
                    jhs.push(Self::spawn_send(tx_clone, data_clone));
                }
                let dropped_receiver_idxs = Self::wait_join_handles(jhs).await;
                txs = Self::filter_sender_by_dropped_receiver_idx(
                    txs.to_owned(),
                    dropped_receiver_idxs,
                );
                Self::inc_success_run(&context).await;
                let is_end = { *(is_end_clone.lock().await) };
                if is_end {
                    break;
                }
            }
            Self::set_state(&context, State::Done).await;
        });
        let join_all = tokio::spawn(async move { tokio::join!(join_event, join_flush) });
        match join_all.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }

    fn add_sender(&mut self, tx: Sender<U>) {
        self.txs.push(tx.into())
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.to_owned()
    }
}

#[macro_export]
macro_rules! collector {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Collector {
                name: $name,
                rx: std::sync::Arc::new(tokio::sync::Mutex::new($rx)),
                txs: vec![],
                config: config,
                collector: std::marker::PhantomData,
                collection: std::marker::PhantomData,
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
        collector!($name, "", $config, $rx, [$( $tx ), *])
    };
}
