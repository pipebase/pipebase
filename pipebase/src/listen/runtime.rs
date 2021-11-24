use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

use super::Listen;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeError, Result, State,
    SubscribeError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::channel;

pub struct Listener<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
}

/// Spawn two tasks
/// * Run listener
/// * Receive data from listener and send downstreams
/// # Parameters
/// * U: output
/// * L: listener
#[async_trait]
impl<'a, U, L, C> Pipe<(), U, L, C> for Listener<'a>
where
    U: Clone + Send + 'static,
    L: Listen<U, C> + 'static,
    C: ConfigInto<L> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        rx: Option<Receiver<()>>,
    ) -> Result<()> {
        assert!(rx.is_none(), "listener {} has invalid upstreams", self.name);
        assert!(!txs.is_empty(), "listener {} has no downstreams", self.name);
        // connect listener
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut listener = config.config_into().await?;
        listener.set_sender(tx0);
        let name = self.name.to_owned();
        let etx = self.etx.clone();
        // start listener
        let join_listener = tokio::spawn(async move {
            match listener.run().await {
                Ok(_) => info!("listener exit ..."),
                Err(err) => {
                    error!("listener {} exit with error '{:#?}'", name, err);
                    send_pipe_error(etx.as_ref(), PipeError::new(name, err)).await
                }
            };
        });
        // start event loop
        let mut txs = senders_as_map(txs);
        let context = self.context.clone();
        let name = self.name.to_owned();
        let join_send_loop = tokio::spawn(async move {
            info!("listener {} run ...", name);
            loop {
                context.set_state(State::Receive);
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        break;
                    }
                    false => (),
                }
                let u = match rx0.recv().await {
                    Some(u) => u,
                    None => {
                        break;
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
                assert!(u_replicas.is_empty(), "replica left over");
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
                context.inc_total_run();
            }
            info!("listener {} exit ...", name);
            context.set_state(State::Done);
        });
        // join listener and loop
        match tokio::spawn(async move { tokio::join!(join_listener, join_send_loop) }).await {
            Ok(_) => (),
            Err(err) => {
                error!("listener join error {:#?}", err)
            }
        }
        Ok(())
    }
}

impl<'a> HasContext for Listener<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Listener<'a> {
    pub fn new(name: &'a str) -> Self {
        Listener {
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

impl<'a> SubscribeError for Listener<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
    }
}

#[macro_export]
macro_rules! listener {
    (
        $name:expr
    ) => {{
        Listener::new($name)
    }};
}
