mod hash;
mod random;
mod roundrobin;

pub use hash::*;
pub use random::*;
pub use roundrobin::*;

use crate::{
    filter_senders_by_indices, senders_as_map, spawn_send, wait_join_handles, ConfigInto,
    FromConfig, HasContext, Pipe,
};
use async_trait::async_trait;
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::context::{Context, State};
use crate::error::Result;

pub trait Select<T, C>: Send + Sync + FromConfig<C> {
    fn select(&mut self, t: &T, candidates: &[&usize]) -> Vec<usize>;
}

pub struct Selector<'a> {
    name: &'a str,
    context: Arc<Context>,
}

#[async_trait]
impl<'a, T, S, C> Pipe<T, T, S, C> for Selector<'a>
where
    T: Clone + Send + 'static,
    S: Select<T, C>,
    C: ConfigInto<S> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<T>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let mut selector = config.config_into().await?;
        let rx = rx.as_mut().unwrap();
        let mut txs = senders_as_map(txs);
        log::info!("selector {} run ...", self.name);
        loop {
            self.context.inc_total_run();
            self.context.set_state(State::Receive);
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    self.context.inc_success_run();
                    break;
                }
                false => (),
            }
            let t = rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    self.context.inc_success_run();
                    break;
                }
            };
            self.context.set_state(State::Send);
            let candidates = txs.keys().collect::<Vec<&usize>>();
            let mut jhs = HashMap::new();
            for i in selector.select(&t, &candidates) {
                let tx = txs.get(&i).unwrap();
                let t_clone = t.to_owned();
                jhs.insert(i, spawn_send(tx.to_owned(), t_clone));
            }
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            self.context.inc_success_run();
        }
        log::info!("selector {} exit ...", self.name);
        self.context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Selector<'a> {
    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Selector<'a> {
    pub fn new(name: &'a str) -> Self {
        Selector {
            name: name,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! selector {
    (
        $name:expr
    ) => {{
        Selector::new($name)
    }};
}
