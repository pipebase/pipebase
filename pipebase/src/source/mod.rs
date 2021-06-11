mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::Pipe;
use std::sync::Arc;

#[async_trait]
pub trait Poll<T>: Send + Sync {
    async fn poll(
        &mut self,
    ) -> std::result::Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Source<'a, T> {
    pub name: &'a str,
    pub txs: Vec<Arc<Sender<T>>>,
    pub poller: Box<dyn Poll<T>>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Clone + Send + 'static> Pipe<T> for Source<'a, T> {
    async fn run(&mut self) -> Result<()> {
        loop {
            Self::inc_total_run(self.context.clone()).await;
            Self::set_state(self.context.clone(), State::Poll).await;
            let t = self.poller.poll().await;
            let t = match t {
                Ok(t) => t,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    continue;
                }
            };
            let t = match t {
                Some(t) => t,
                None => break,
            };
            Self::set_state(self.context.clone(), State::Send).await;
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let t_clone = t.to_owned();
                jhs.push(Self::spawn_send(tx, t_clone));
            }
            match Self::wait_join_handles(jhs).await {
                _ => (),
            }
            Self::inc_success_run(self.context.clone()).await;
        }
        Self::set_state(self.context.clone(), State::Done).await;
        Self::inc_success_run(self.context.clone()).await;
        info!("source {} exit ...", self.name);
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! source {
    (
        $name:expr, $path:expr, $config:ty, $poller:ty, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let poller = <$poller>::from_config(&config).await.unwrap();
            let mut pipe = Source {
                name: $name,
                txs: vec![],
                poller: Box::new(poller),
                context: Default::default()
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}
