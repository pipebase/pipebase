mod bag;
mod set;
pub use bag::*;
pub use set::*;

use std::iter::FromIterator;

use crate::HasContext;
use crate::{
    context::{Context, State},
    error::join_error,
    filter_senders_by_indices, inc_success_run, inc_total_run, senders_as_map, set_state,
    spawn_send, wait_join_handles, ConfigInto, FromConfig, Pipe, Result,
};

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    },
    time::Interval,
};

#[async_trait]
pub trait Collect<T, U, C>: Send + Sync + FromConfig<C>
where
    T: Clone,
    U: FromIterator<T> + Clone,
{
    async fn collect(&mut self, t: &T);
    async fn flush(&mut self) -> U;
    fn get_flush_interval(&self) -> Interval;
}

pub struct Collector<'a> {
    name: &'a str,
    context: Arc<RwLock<Context>>,
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
        mut rx: Option<Receiver<T>>,
        txs: Vec<Sender<U>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let collector: Arc<Mutex<V>> = Arc::new(Mutex::new(config.config_into().await.unwrap()));
        let collector_clone = collector.to_owned();
        let is_end = Arc::new(Mutex::new(false));
        let is_end_clone = is_end.to_owned();
        let join_event = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
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
        let mut txs = senders_as_map(txs);
        let is_end_clone = is_end.to_owned();
        let context = self.get_context();
        let name = self.name.to_owned();
        let join_flush = tokio::spawn(async move {
            let mut interval = {
                let c = collector_clone.lock().await;
                c.get_flush_interval()
            };
            log::info!("collector {} run ...", name);
            loop {
                set_state(&context, State::Receive).await;
                inc_total_run(&context).await;
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        inc_success_run(&context).await;
                        break;
                    }
                    false => (),
                }
                interval.tick().await;
                let mut c = collector_clone.lock().await;
                let data = c.flush().await;
                set_state(&context, State::Send).await;
                let mut jhs = HashMap::new();
                for (idx, tx) in &txs {
                    let tx_clone = tx.to_owned();
                    let data_clone = data.to_owned();
                    jhs.insert(idx.to_owned(), spawn_send(tx_clone, data_clone));
                }
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
                inc_success_run(&context).await;
                let is_end = { *(is_end_clone.lock().await) };
                if is_end {
                    break;
                }
            }
            log::info!("collector {} exit ...", name);
            set_state(&context, State::Done).await;
        });
        let join_all = tokio::spawn(async move { tokio::join!(join_event, join_flush) });
        match join_all.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }
}

impl<'a> HasContext for Collector<'a> {
    fn get_context(&self) -> Arc<RwLock<Context>> {
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
