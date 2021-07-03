mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::context::{Context, State};
use crate::error::Result;
use crate::HasContext;
use crate::{
    filter_senders_by_indices, senders_as_map, spawn_send, wait_join_handles, ConfigInto,
    FromConfig, Pipe,
};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    async fn poll(&mut self) -> anyhow::Result<Option<T>>;
}

pub struct Poller<'a> {
    name: &'a str,
    context: Arc<Context>,
}

#[async_trait]
impl<'a, U, P, C> Pipe<(), U, P, C> for Poller<'a>
where
    U: Clone + Send + 'static,
    P: Poll<U, C>,
    C: ConfigInto<P> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        rx: Option<Receiver<()>>,
    ) -> Result<()> {
        assert!(rx.is_none());
        let mut poller = config.config_into().await.unwrap();
        let mut txs = senders_as_map(txs);
        info!("source {} run ...", self.name);
        loop {
            self.context.inc_total_run();
            self.context.set_state(State::Poll);
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    // Self::inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let u = poller.poll().await;
            let u = match u {
                Ok(u) => u,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    break;
                }
            };
            let u = match u {
                Some(u) => u,
                None => {
                    self.context.inc_success_run();
                    break;
                }
            };
            self.context.set_state(State::Send);
            let mut jhs = HashMap::new();
            for (idx, tx) in &txs {
                let u_clone = u.to_owned();
                jhs.insert(idx.to_owned(), spawn_send(tx.to_owned(), u_clone));
            }
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            self.context.inc_success_run();
        }
        info!("source {} exit ...", self.name);
        self.context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Poller<'a> {
    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Poller<'a> {
    pub fn new(name: &'a str) -> Self {
        Poller {
            name: name,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! poller {
    (
        $name:expr
    ) => {{
        Poller::new($name)
    }};
}
