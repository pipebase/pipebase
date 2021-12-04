use std::sync::atomic::{AtomicBool, Ordering};

use super::Collect;
use crate::common::{
    filter_senders_by_indices, replicate, send_pipe_error, senders_as_map, spawn_send,
    wait_join_handles, ConfigInto, Context, HasContext, Pipe, PipeError, Result, State,
    SubscribeError,
};

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    sync::{
        mpsc::{error::SendError, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{error, info};

pub struct Collector<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
}

/// Spawn two tasks
/// * Run collector and collect items in buffer
/// * Flush collector in period and send data to downstreams
/// # Parameters
/// * T: input
/// * U: output
/// * V: collector
#[async_trait]
impl<'a, T, U, V, C> Pipe<T, U, V, C> for Collector<'a>
where
    T: Clone + Send + Sync + 'static,
    U: Clone + Send + 'static,
    V: Collect<T, U, C> + 'static,
    C: ConfigInto<V> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some(), "collector '{}' has no upstreams", self.name);
        assert!(
            !txs.is_empty(),
            "collector '{}' has no downstreams",
            self.name
        );
        let collector: Arc<Mutex<V>> = Arc::new(Mutex::new(config.config_into().await?));
        let collector_clone = collector.to_owned();
        let exit_c = Arc::new(AtomicBool::new(false));
        let exit_c_clone = exit_c.to_owned();
        let exit_f = Arc::new(AtomicBool::new(false));
        let exit_f_clone = exit_f.to_owned();
        let name = self.name.to_owned();
        let etx = self.etx.clone();
        let join_collect = tokio::spawn(async move {
            let rx = rx.as_mut().unwrap();
            info!(
                name = name.as_str(),
                ty = "collector",
                thread = "collect",
                "run ..."
            );
            loop {
                if exit_f_clone.load(Ordering::Acquire) {
                    break;
                }
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => {
                        exit_c.store(true, Ordering::Release);
                        break;
                    }
                };
                let mut c = collector_clone.lock().await;
                match c.collect(t).await {
                    Ok(()) => continue,
                    Err(err) => {
                        error!(
                            name = name.as_str(),
                            ty = "collector",
                            thread = "collect",
                            "error '{}' ...",
                            err
                        );
                        send_pipe_error(etx.as_ref(), PipeError::new(name.to_owned(), err)).await;
                    }
                }
            }
            info!(
                name = name.as_str(),
                ty = "collector",
                thread = "collect",
                "exit ..."
            );
        });
        let mut txs = senders_as_map(txs);
        let context = self.get_context();
        let name = self.name.to_owned();
        let etx = self.etx.clone();
        let join_flush = tokio::spawn(async move {
            info!(
                name = name.as_str(),
                ty = "collector",
                thread = "flush",
                "run ..."
            );
            let mut interval = {
                let c = collector.lock().await;
                c.get_flush_interval()
            };
            loop {
                context.set_state(State::Receive);
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        break;
                    }
                    false => (),
                }
                interval.tick().await;
                let u = {
                    let mut c = collector.lock().await;
                    let u = match c.flush().await {
                        Ok(u) => u,
                        Err(err) => {
                            error!(
                                name = name.as_str(),
                                ty = "collector",
                                thread = "flush",
                                "error '{}' ...",
                                err
                            );
                            context.inc_failure_run();
                            context.inc_total_run();
                            send_pipe_error(etx.as_ref(), PipeError::new(name.to_owned(), err))
                                .await;
                            continue;
                        }
                    };
                    match u {
                        Some(u) => u,
                        None => continue,
                    }
                };
                context.set_state(State::Send);
                let mut u_replicas = replicate(u, txs.len());
                assert!(!u_replicas.is_empty(), "empty replicas");
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
                if exit_c_clone.load(Ordering::Acquire) {
                    break;
                }
            }
            info!(
                name = name.as_str(),
                ty = "collector",
                thread = "flush",
                "exit ..."
            );
            exit_f.store(true, Ordering::Release);
            context.set_state(State::Done);
        });
        match tokio::spawn(async move { tokio::join!(join_collect, join_flush) }).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    name = self.name,
                    ty = "collector",
                    thread = "join",
                    "join error '{:#?}'",
                    err
                )
            }
        }
        Ok(())
    }
}

impl<'a> HasContext for Collector<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.to_owned()
    }
}

impl<'a> Collector<'a> {
    pub fn new(name: &'a str) -> Self {
        Collector {
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

impl<'a> SubscribeError for Collector<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
    }
}

#[macro_export]
macro_rules! collector {
    (
        $name:expr
    ) => {{
        Collector::new($name)
    }};
}
