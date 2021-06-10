mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use tokio::sync::mpsc::Sender;

use crate::error::{join_error, Result};
use crate::{spawn_send, wait_join_handle};
use std::sync::Arc;

#[async_trait]
pub trait Poll<T>: Send {
    async fn poll(
        &mut self,
    ) -> std::result::Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Source<'a, T> {
    pub name: &'a str,
    pub txs: Vec<Arc<Sender<T>>>,
    pub poller: Box<dyn Poll<T>>,
}

impl<'a, T: Clone + Send + 'static> Source<'a, T> {
    pub async fn run(&mut self) -> Result<()> {
        loop {
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
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let t_clone = t.to_owned();
                jhs.push(spawn_send!(tx, t_clone, jhs));
            }
            for jh in jhs {
                wait_join_handle!(jh)
            }
        }
        info!("source {} exit ...", self.name);
        Ok(())
    }

    pub fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(Arc::new(tx));
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
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}
