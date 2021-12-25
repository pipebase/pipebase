use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

use super::Listen;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeChannels, PipeError, Result,
    State, SubscribeError,
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
    async fn run(self, config: C, channels: PipeChannels<(), U>) -> Result<()> {
        let name = self.name;
        let context = self.context;
        let etx = self.etx;
        let (rx, txs) = channels.into_channels();
        assert!(rx.is_none(), "listener '{}' has invalid upstreams", name);
        assert!(!txs.is_empty(), "listener '{}' has no downstreams", name);
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut listener = config.config_into().await?;
        listener.set_sender(tx0);
        let pipe_name = name.to_owned();
        // start listen
        let join_listen = tokio::spawn(async move {
            info!(
                name = pipe_name.as_str(),
                ty = "listener",
                thread = "listen",
                "run ..."
            );
            match listener.run().await {
                Ok(_) => info!(
                    name = pipe_name.as_str(),
                    ty = "listener",
                    thread = "listen",
                    "exit ..."
                ),
                Err(err) => {
                    error!(
                        name = pipe_name.as_str(),
                        ty = "listener",
                        thread = "listen",
                        "exit with error '{:#?}'",
                        err
                    );
                    send_pipe_error(etx.as_ref(), PipeError::new(pipe_name, err)).await
                }
            };
        });
        // start send
        let mut txs = senders_as_map(txs);
        let pipe_name = name.to_owned();
        let join_send = tokio::spawn(async move {
            info!(
                name = pipe_name.as_str(),
                ty = "listener",
                thread = "send",
                "run ..."
            );
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
                assert!(u_replicas.is_empty(), "replica leftover");
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
                context.inc_total_run();
            }
            info!(
                name = pipe_name.as_str(),
                ty = "listener",
                thread = "send",
                "exit ..."
            );
            context.set_state(State::Done);
        });
        // join listen and send
        let pipe_name = name.to_owned();
        match tokio::spawn(async move { tokio::join!(join_listen, join_send) }).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    name = pipe_name.as_str(),
                    ty = "listener",
                    thread = "join",
                    "join error {:#?}",
                    err
                )
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
