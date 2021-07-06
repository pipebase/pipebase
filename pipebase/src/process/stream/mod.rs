mod file;
mod iterator;

pub use file::*;
pub use iterator::*;

use crate::context::{Context, State};
use crate::{filter_senders_by_indices, senders_as_map, spawn_send, wait_join_handles};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{ConfigInto, FromConfig, HasContext, Pipe};

#[async_trait]
pub trait Stream<T, U, C>: Send + Sync + FromConfig<C>
where
    U: Send + 'static,
{
    async fn stream(&mut self, t: T) -> anyhow::Result<()>;
    fn set_sender(&mut self, sender: Sender<U>);
}

pub struct Streamer<'a> {
    name: &'a str,
    context: Arc<Context>,
}

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
    ) -> crate::error::Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut streamer = config.config_into().await?;
        streamer.set_sender(tx0);
        let name = self.name.to_owned();
        let streamer_loop = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
            log::info!("streamer {} run ...", name);
            loop {
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => break,
                };
                match streamer.stream(t).await {
                    Ok(_) => continue,
                    Err(err) => {
                        log::error!("streamer error {}", err);
                        break;
                    }
                }
            }
            log::info!("streamer {} exit ...", name);
        });
        let mut txs = senders_as_map(txs);
        let context = self.context.clone();
        let sender_loop = tokio::spawn(async move {
            loop {
                context.inc_total_run();
                context.set_state(State::Receive);
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        context.inc_success_run();
                        break;
                    }
                    false => (),
                }
                let u = match rx0.recv().await {
                    Some(u) => u,
                    None => {
                        context.inc_success_run();
                        // EOF, streamer loop break
                        break;
                    }
                };
                context.set_state(State::Send);
                let mut jhs = HashMap::new();
                for (idx, tx) in &txs {
                    let u_clone: U = u.to_owned();
                    jhs.insert(idx.to_owned(), spawn_send(tx.clone(), u_clone));
                }
                let drop_sender_indices = wait_join_handles(jhs).await;
                filter_senders_by_indices(&mut txs, drop_sender_indices);
                context.inc_success_run();
            }
            context.set_state(State::Done);
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
    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Streamer<'a> {
    pub fn new(name: &'a str) -> Self {
        Streamer {
            name: name,
            context: Default::default(),
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
