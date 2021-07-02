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
    filter_senders_by_indices, inc_success_run, inc_total_run, senders_as_map, set_state,
    spawn_send, wait_join_handles, ConfigInto, FromConfig, HasContext, Pipe,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Map<T, U, C>: Send + Sync + FromConfig<C> {
    async fn map(&mut self, data: T) -> anyhow::Result<U>;
}

pub struct Mapper<'a> {
    name: &'a str,
    context: Arc<RwLock<Context>>,
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
        mut rx: Option<Receiver<T>>,
        txs: Vec<Sender<U>>,
    ) -> Result<()> {
        assert!(rx.is_some());
        assert!(!txs.is_empty());
        let mut mapper = config.config_into().await.unwrap();
        let mut txs = senders_as_map(txs);
        let rx = rx.as_mut().unwrap();
        log::info!("mapper {} run ...", self.name);
        loop {
            inc_total_run(&self.context).await;
            set_state(&self.context, State::Receive).await;
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let t = rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    inc_success_run(&self.context).await;
                    break;
                }
            };
            set_state(&self.context, State::Process).await;
            let u = match mapper.map(t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    break;
                }
            };
            set_state(&self.context, State::Send).await;
            let mut jhs = HashMap::new();
            for (idx, tx) in &txs {
                let u_clone: U = u.to_owned();
                jhs.insert(idx.to_owned(), spawn_send(tx.to_owned(), u_clone));
            }
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            inc_success_run(&self.context).await;
        }
        log::info!("mapper {} exit ...", self.name);
        set_state(&self.context, State::Done).await;
        Ok(())
    }
}

impl<'a> HasContext for Mapper<'a> {
    fn get_context(&self) -> Arc<RwLock<Context>> {
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
