use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinHandle;

use crate::{
    filter_senders_by_indices, replicate, senders_as_map, spawn_send, wait_join_handles,
    ConfigInto, Context, HasContext, Pipe, Result, Select, State,
};
use async_trait::async_trait;
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Selector<'a> {
    name: &'a str,
    context: Arc<Context>,
}

/// Start loop
/// * Select downstreams
/// * Send data to selected downstreams
/// # Parameters
/// * T: input/output
/// * S: selector
#[async_trait]
impl<'a, T, S, C> Pipe<T, T, S, C> for Selector<'a>
where
    T: Clone + Send + 'static,
    S: Select<T, C>,
    C: ConfigInto<S> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<T>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some(), "selector {} has no upstreams", self.name);
        assert!(!txs.is_empty(), "selector {} has no downstreams", self.name);
        let mut selector = config.config_into().await?;
        let rx = rx.as_mut().unwrap();
        let mut txs = senders_as_map(txs);
        log::info!("selector {} run ...", self.name);
        loop {
            self.context.set_state(State::Receive);
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
            self.context.set_state(State::Send);
            let candidates = txs.keys().collect::<Vec<&usize>>();
            let selected = selector.select(&t, &candidates);
            let mut t_replicas = replicate(t, selected.len());
            let jhs: HashMap<usize, JoinHandle<core::result::Result<(), SendError<T>>>> = selected
                .into_iter()
                .map(|i| {
                    (
                        i,
                        spawn_send(
                            txs.get(&i).expect("sender").to_owned(),
                            t_replicas.pop().expect("no replica left"),
                        ),
                    )
                })
                .collect();
            assert!(t_replicas.is_empty(), "replica left over");
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            self.context.inc_total_run();
        }
        log::info!("selector {} exit ...", self.name);
        self.context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Selector<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Selector<'a> {
    pub fn new(name: &'a str) -> Self {
        Selector {
            name: name,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! selector {
    (
        $name:expr
    ) => {{
        Selector::new($name)
    }};
}
