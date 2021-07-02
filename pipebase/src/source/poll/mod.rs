mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::HasContext;
use crate::{
    filter_senders_by_indices, inc_success_run, inc_total_run, senders_as_map, set_state,
    spawn_send, wait_join_handles, ConfigInto, FromConfig, Pipe,
};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Poll<T, C>: Send + Sync + FromConfig<C> {
    async fn poll(&mut self) -> anyhow::Result<Option<T>>;
}

pub struct Poller<'a> {
    name: &'a str,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, U, P, C> Pipe<(), U, P, C> for Poller<'a>
where
    U: Clone + Send + 'static,
    P: Poll<U, C>,
    C: ConfigInto<P> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<U>>,
        rx: Option<Receiver<()>>,
    ) -> Result<()> {
        assert!(rx.is_none());
        let mut poller = config.config_into().await.unwrap();
        let mut txs = senders_as_map(txs);
        info!("source {} run ...", self.name);
        loop {
            inc_total_run(&self.context).await;
            set_state(&self.context, State::Poll).await;
            // if all receiver dropped, sender drop as well
            match txs.is_empty() {
                true => {
                    // Self::inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let u = poller.poll().await;
            let u = match u {
                Ok(u) => u,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    break;
                }
            };
            let u = match u {
                Some(u) => u,
                None => {
                    inc_success_run(&self.context).await;
                    break;
                }
            };
            set_state(&self.context, State::Send).await;
            let mut jhs = HashMap::new();
            for (idx, tx) in &txs {
                let u_clone = u.to_owned();
                jhs.insert(idx.to_owned(), spawn_send(tx.to_owned(), u_clone));
            }
            let drop_sender_indices = wait_join_handles(jhs).await;
            filter_senders_by_indices(&mut txs, drop_sender_indices);
            inc_success_run(&self.context).await;
        }
        info!("source {} exit ...", self.name);
        set_state(&self.context, State::Done).await;
        Ok(())
    }
}

impl<'a> HasContext for Poller<'a> {
    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

impl<'a> Poller<'a> {
    pub fn new(name: &'a str) -> Self {
        Poller {
            name: name,
            context: Default::default(),
        }
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
