use async_trait::async_trait;
use tokio::{
    sync::mpsc::{error::SendError, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};
use tracing::{error, info};

use super::{Poll, PollResponse};
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeError, Result, State,
    SubscribeError,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Poller<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
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
    async fn run(self, config: C, txs: Vec<Sender<U>>, rx: Option<Receiver<()>>) -> Result<()> {
        let name = self.name;
        let context = self.context;
        let etx = self.etx;
        assert!(rx.is_none(), "poller '{}' has invalid upstreams", name);
        assert!(!txs.is_empty(), "poller '{}' has no downstreams", name);
        let mut poller = config.config_into().await?;
        let mut txs = senders_as_map(txs);
        info!(name = name, ty = "poller", "run ...");
        let delay = poller.get_initial_delay();
        let mut interval = poller.get_interval();
        // initial delay
        sleep(delay).await;
        // first tick start immediately
        interval.tick().await;
        context.set_state(State::Poll);
        loop {
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    break;
                }
                false => (),
            }
            let resp = poller.poll().await;
            let resp = match resp {
                Ok(resp) => resp,
                Err(err) => {
                    error!(name = name, ty = "poller", "error '{:#?}'", err);
                    context.inc_total_run();
                    context.inc_failure_run();
                    // wait for next poll period
                    send_pipe_error(etx.as_ref(), PipeError::new(name.to_owned(), err)).await;
                    interval.tick().await;
                    continue;
                }
            };
            let resp = match resp {
                PollResponse::Exit => break,
                PollResponse::PollResult(resp) => resp,
            };
            let u = match resp {
                Some(u) => u,
                None => {
                    // wait for next poll period
                    interval.tick().await;
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
            // wait for next poll period
            context.set_state(State::Poll);
            interval.tick().await;
        }
        info!(name = name, ty = "poller", "exit ...");
        context.set_state(State::Done);
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
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

impl<'a> SubscribeError for Poller<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
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
