mod echo;
mod field;
mod filter;
mod project;

use std::fmt::Debug;

use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::{join_error, Result};
use crate::{spawn_send, wait_join_handle};
use std::sync::Arc;

#[async_trait]
pub trait Procedure<T, U>: Send {
    async fn process(&mut self, data: &T) -> std::result::Result<U, Box<dyn std::error::Error>>;
}

pub struct Process<'a, T, U> {
    name: &'a str,
    rx: Receiver<T>,
    txs: Vec<Arc<Sender<U>>>,
    procedure: Box<dyn Procedure<T, U>>,
}

impl<'a, T, U: Clone + Debug + Send + 'static> Process<'a, T, U> {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            let u = match self.procedure.process(&t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    continue;
                }
            };
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let u_clone = u.to_owned();
                jhs.push(spawn_send!(tx, u_clone, jhs));
            }
            for jh in jhs {
                wait_join_handle!(jh)
            }
        }
        Ok(())
    }

    pub fn add_sender(&mut self, tx: Sender<U>) {
        self.txs.push(Arc::new(tx));
    }
}

#[macro_export]
macro_rules! process {
    (
        $name:expr, $path:expr, $config:ty, $procedure:ty, $rx: ident, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let procedure = <$procedure>::from_config(&config).await.unwrap();
            let mut pipe = Process {
                name: $name,
                rx: $rx,
                txs: vec![],
                procedure: Box::new(procedure),
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}
