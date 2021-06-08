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
    pub tx: Sender<T>,
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

            match self.tx.send(t.clone()).await {
                Ok(_) => continue,
                Err(err) => {
                    error!("source send error {:#?}", err.to_string())
                }
            }
        }
        info!("source {} exit ...", self.name)
    }
}
