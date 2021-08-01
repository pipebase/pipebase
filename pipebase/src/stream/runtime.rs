use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};
use tokio::task::JoinHandle;

use super::Stream;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeError, Result, State,
    SubscribeError,
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
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some(), "streamer {} has no upstreams", self.name);
        assert!(!txs.is_empty(), "streamer {} has no downstreams", self.name);
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut streamer = config.config_into().await?;
        streamer.set_sender(tx0);
        let name = self.name.to_owned();
        let context = self.context.clone();
        let etx = self.etx.clone();
        let streamer_loop = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
            log::info!("streamer {} run ...", name);
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
                        log::error!("streamer {} error '{:#?}'", name, err);
                        send_pipe_error(etx.as_ref(), PipeError::new(name.to_owned(), err)).await;
                        context.inc_failure_run();
                    }
                }
                context.inc_total_run();
            }
            log::info!("streamer {} exit ...", name);
            context.set_state(State::Done);
        });
        let mut txs = senders_as_map(txs);
        let sender_loop = tokio::spawn(async move {
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
                assert!(u_replicas.is_empty(), "replica left over");
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
            }
        });
        // join listener and loop
        match tokio::spawn(async move { tokio::join!(streamer_loop, sender_loop) }).await {
            Ok(_) => (),
            Err(err) => {
                log::error!("streamer join error {:#?}", err)
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
            name: name,
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
