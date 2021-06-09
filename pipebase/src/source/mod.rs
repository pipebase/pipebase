mod timer;

pub use timer::*;

use async_trait::async_trait;
use log::{error, info};
use std::error::Error;
use std::result::Result;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait Poll<T>: Send + Sync {
    async fn poll(&mut self) -> Result<Option<T>, Box<dyn Error + Send + Sync>>;
}

pub struct Source<'a, T> {
    pub name: &'a str,
    pub txs: Vec<Sender<T>>,
    pub poller: Box<dyn Poll<T>>,
}

impl<'a, T: Clone> Source<'a, T> {
    pub async fn run(&mut self) {
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
            for tx in self.txs.as_mut_slice() {
                match tx.send(t.clone()).await {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("source send error {:#?}", err.to_string())
                    }
                }
            }
        }
        info!("source {} exit ...", self.name)
    }

    pub fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(tx);
    }
}

#[macro_export]
macro_rules! source {
    (
        $name:expr, $path:expr, $config:ty, $poller:ty, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect("valid config file");
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
