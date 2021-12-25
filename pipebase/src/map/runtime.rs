use std::fmt::Debug;

use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

use super::Map;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeChannels, PipeError, Result,
    State, SubscribeError,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Mapper<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
}

/// Start loop
/// * Receive and map data
/// * Send mapper's output to downstrem
/// # Parameters
/// * T: input
/// * U: output
/// * M: mapper
#[async_trait]
impl<'a, T, U, M, C> Pipe<T, U, M, C> for Mapper<'a>
where
    T: Send + Sync + 'static,
    U: Clone + Debug + Send + 'static,
    M: Map<T, U, C>,
    C: ConfigInto<M> + Send + Sync + 'static,
{
    async fn run(self, config: C, channels: PipeChannels<T, U>) -> Result<()> {
        let name = self.name;
        let context = self.context;
        let etx = self.etx;
        let (mut rx, txs) = channels.into_channels();
        assert!(rx.is_some(), "mapper '{}' has no upstreams", name);
        assert!(!txs.is_empty(), "mapper '{}' has no downstreams", name);
        let mut mapper = config.config_into().await?;
        let mut txs = senders_as_map(txs);
        let rx = rx.as_mut().unwrap();
        info!(name = name, ty = "mapper", "run ...");
        loop {
            context.set_state(State::Receive);
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    break;
                }
                false => (),
            }
            let t = rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    break;
                }
            };
            context.set_state(State::Map);
            let u = match mapper.map(t).await {
                Ok(u) => u,
                Err(err) => {
                    error!(name = name, ty = "mapper", "error '{:#?}'", err);
                    context.inc_total_run();
                    context.inc_failure_run();
                    send_pipe_error(etx.as_ref(), PipeError::new(name.to_owned(), err)).await;
                    continue;
                }
            };
            context.set_state(State::Send);
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
            assert!(u_replicas.is_empty(), "replica leftover");
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            context.inc_total_run();
        }
        info!(name = name, ty = "mapper", "exit ...");
        context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Mapper<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Mapper<'a> {
    pub fn new(name: &'a str) -> Self {
        Mapper {
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

impl<'a> SubscribeError for Mapper<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
    }
}

#[macro_export]
macro_rules! mapper {
    (
        $name:expr
    ) => {{
        Mapper::new($name)
    }};
}
