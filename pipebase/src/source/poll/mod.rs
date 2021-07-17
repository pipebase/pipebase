mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::context::{Context, State};
use crate::error::Result;
use crate::HasContext;
use crate::{
    filter_senders_by_indices, replicate, senders_as_map, spawn_send, wait_join_handles,
    ConfigInto, FromConfig, Pipe,
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

/// Start loop
/// * Poll data from external
/// * Send data to downstreams
/// # Parameters
/// * U: output
/// * P: poller
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
        assert!(rx.is_none(), "poller {} has invalid upstreams", self.name);
        assert!(!txs.is_empty(), "poller {} has no downstreams", self.name);
        let mut poller = config.config_into().await?;
        let mut txs = senders_as_map(txs);
        info!("source {} run ...", self.name);
        loop {
            self.context.set_state(State::Poll);
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    break;
                }
                false => (),
            }
            let u = poller.poll().await;
            let u = match u {
                Ok(u) => u,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    self.context.inc_total_run();
                    self.context.inc_failure_run();
                    continue;
                }
            };
            let u = match u {
                Some(u) => u,
                None => {
                    break;
                }
            };
            self.context.set_state(State::Send);
            let mut u_replicas = replicate(u, txs.len());
            let jhs: HashMap<usize, JoinHandle<core::result::Result<(), SendError<U>>>> = txs
                .iter()
                .map(|(idx, tx)| {
                    (
                        idx.to_owned(),
                        spawn_send(tx.to_owned(), u_replicas.pop().expect("no replica left")),
                    )
                })
                .collect();
            assert!(u_replicas.is_empty(), "replica left over");
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            self.context.inc_total_run();
        }
        info!("source {} exit ...", self.name);
        self.context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Poller<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

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
