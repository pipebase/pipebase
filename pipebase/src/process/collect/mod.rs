mod bag;
mod set;
pub use bag::*;
pub use set::*;

use std::iter::FromIterator;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::HasContext;
use crate::{
    context::{Context, State},
    error::join_error,
    filter_senders_by_indices, senders_as_map, spawn_send, wait_join_handles, ConfigInto,
    FromConfig, Pipe, Result,
};

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::Interval,
};

#[async_trait]
pub trait Collect<T, U, C>: Send + FromConfig<C>
where
    U: FromIterator<T> + Clone,
{
    async fn collect(&mut self, t: T);
    async fn flush(&mut self) -> U;
    fn get_flush_interval(&self) -> Interval;
}

pub struct Collector<'a> {
    name: &'a str,
    context: Arc<Context>,
}

#[async_trait]
impl<'a, T, U, V, C> Pipe<T, U, V, C> for Collector<'a>
where
    T: Clone + Send + Sync + 'static,
    U: FromIterator<T> + Clone + Send + 'static,
    V: Collect<T, U, C> + 'static,
    C: ConfigInto<V> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let collector: Arc<Mutex<V>> = Arc::new(Mutex::new(config.config_into().await.unwrap()));
        let collector_clone = collector.to_owned();
        let is_end = Arc::new(AtomicBool::new(false));
        let is_end_clone = is_end.to_owned();
        let join_event = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
            loop {
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => {
                        is_end_clone.store(true, Ordering::Release);
                        break;
                    }
                };
                let mut c = collector_clone.lock().await;
                (*c).collect(t).await;
            }
        });
        let mut txs = senders_as_map(txs);
        let context = self.get_context();
        let name = self.name.to_owned();
        let join_flush = tokio::spawn(async move {
            let mut interval = {
                let c = collector.lock().await;
                c.get_flush_interval()
            };
            log::info!("collector {} run ...", name);
            loop {
                context.set_state(State::Receive);
                context.inc_total_run();
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        context.inc_success_run();
                        break;
                    }
                    false => (),
                }
                interval.tick().await;
                let data = {
                    let mut c = collector.lock().await;
                    c.flush().await
                };
                context.set_state(State::Send);
                let mut jhs = HashMap::new();
                for (idx, tx) in &txs {
                    let tx_clone = tx.to_owned();
                    let data_clone = data.to_owned();
                    jhs.insert(idx.to_owned(), spawn_send(tx_clone, data_clone));
                }
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
                context.inc_success_run();
                if is_end.load(Ordering::Acquire) {
                    break;
                }
            }
            log::info!("collector {} exit ...", name);
            context.set_state(State::Done);
        });
        let join_all = tokio::spawn(async move { tokio::join!(join_event, join_flush) });
        match join_all.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }
}

impl<'a> HasContext for Collector<'a> {
    fn get_context(&self) -> Arc<Context> {
        self.context.to_owned()
    }
}

impl<'a> Collector<'a> {
    pub fn new(name: &'a str) -> Self {
        Collector {
            name: name,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! collector {
    (
        $name:expr
    ) => {{
        Collector::new($name)
    }};
}
