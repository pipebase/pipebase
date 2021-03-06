use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, error::SendError, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

use super::Stream;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeChannels, PipeError, Result,
    State, SubscribeError,
};

pub struct Streamer<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
}

/// Spawn two tasks
/// * Run streamer
/// * Receive data from streamer and send downstreams
/// # Parameters
/// * T: input
/// * U: output
/// * S: streamer
#[async_trait]
impl<'a, T, U, S, C> Pipe<T, U, S, C> for Streamer<'a>
where
    T: Send + 'static,
    U: Clone + Send + 'static,
    S: Stream<T, U, C> + 'static,
    C: ConfigInto<S> + Send + Sync + 'static,
{
    async fn run(self, config: C, channels: PipeChannels<T, U>) -> Result<()> {
        let name = self.name;
        let context = self.context;
        let etx = self.etx;
        let (mut rx, txs) = channels.into_channels();
        assert!(rx.is_some(), "streamer '{}' has no upstreams", name);
        assert!(!txs.is_empty(), "streamer '{}' has no downstreams", name);
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut streamer = config.config_into().await?;
        streamer.set_sender(tx0);
        let pipe_name = name.to_owned();
        let join_stream = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
            info!(
                name = pipe_name.as_str(),
                ty = "streamer",
                thread = "stream",
                "run ..."
            );
            loop {
                context.set_state(State::Receive);
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => break,
                };
                context.set_state(State::Send);
                match streamer.stream(t).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!(
                            name = pipe_name.as_str(),
                            ty = "streamer",
                            thread = "stream",
                            "error '{:#?}'",
                            err
                        );
                        send_pipe_error(etx.as_ref(), PipeError::new(pipe_name.clone(), err)).await;
                        context.inc_failure_run();
                    }
                }
                context.inc_total_run();
            }
            info!(
                name = pipe_name.as_str(),
                ty = "streamer",
                thread = "stream",
                "exit ..."
            );
            context.set_state(State::Done);
        });
        let mut txs = senders_as_map(txs);
        let pipe_name = name.to_owned();
        // start send
        let join_send = tokio::spawn(async move {
            info!(
                name = pipe_name.as_str(),
                ty = "streamer",
                thread = "send",
                "run ..."
            );
            loop {
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
            }
            info!(
                name = pipe_name.as_str(),
                ty = "streamer",
                thread = "send",
                "exit ..."
            );
        });
        // join stream and send
        let pipe_name = name.to_owned();
        match tokio::spawn(async move { tokio::join!(join_stream, join_send) }).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    name = pipe_name.as_str(),
                    ty = "streamer",
                    thread = "join",
                    "join error '{:#?}'",
                    err
                )
            }
        }
        Ok(())
    }
}

impl<'a> HasContext for Streamer<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> SubscribeError for Streamer<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
    }
}

impl<'a> Streamer<'a> {
    pub fn new(name: &'a str) -> Self {
        Streamer {
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

#[macro_export]
macro_rules! streamer {
    (
        $name:expr
    ) => {{
        Streamer::new($name)
    }};
}
