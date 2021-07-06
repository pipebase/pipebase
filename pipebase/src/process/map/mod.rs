mod aggregate;
mod echo;
mod field;
mod filter;
mod project;
mod split;

pub use aggregate::*;
pub use echo::*;
pub use field::*;
pub use filter::*;
pub use project::*;
pub use split::*;

use std::fmt::Debug;

use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::context::{Context, State};
use crate::error::Result;
use crate::{
    filter_senders_by_indices, senders_as_map, spawn_send, wait_join_handles, ConfigInto,
    FromConfig, HasContext, Pipe,
};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: T) -> anyhow::Result<U>;
}

pub struct Mapper<'a> {
    name: &'a str,
    context: Arc<Context>,
}

#[async_trait]
impl<'a, T, U, M, C> Pipe<T, U, M, C> for Mapper<'a>
where
    T: Send + Sync + 'static,
    U: Clone + Debug + Send + 'static,
    M: Map<T, U, C>,
    C: ConfigInto<M> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let mut mapper = config.config_into().await?;
        let mut txs = senders_as_map(txs);
        let rx = rx.as_mut().unwrap();
        log::info!("mapper {} run ...", self.name);
        loop {
            self.context.inc_total_run();
            self.context.set_state(State::Receive);
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    self.context.inc_success_run();
                    break;
                }
                false => (),
            }
            let t = rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    self.context.inc_success_run();
                    break;
                }
            };
            self.context.set_state(State::Process);
            let u = match mapper.map(t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    break;
                }
            };
            self.context.set_state(State::Send);
            let mut jhs = HashMap::new();
            for (idx, tx) in &txs {
                let u_clone: U = u.to_owned();
                jhs.insert(idx.to_owned(), spawn_send(tx.to_owned(), u_clone));
            }
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            self.context.inc_success_run();
        }
        log::info!("mapper {} exit ...", self.name);
        self.context.set_state(State::Done);
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
            name: name,
            context: Default::default(),
        }
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
