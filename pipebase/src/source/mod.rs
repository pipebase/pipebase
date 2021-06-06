mod timer;

use crate::error::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait Poll<T>: Send + Sync {
    async fn poll(&mut self) -> Option<Result<T>>;
}

pub struct Source<'a, T> {
    name: &'a str,
    tx: Sender<T>,
    p: Box<dyn Poll<T>>,
}

impl<'a, T: Clone> Source<'a, T> {
    pub async fn run(&mut self) {
        loop {
            let t = self.p.poll().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            let t = match t {
                Ok(t) => t,
                Err(e) => {
                    error!("{} poll error {:#?}", self.name, e);
                    continue;
                }
            };
            match self.tx.send(t).await {
                Ok(_) => continue,
                Err(err) => {
                    error!("source send error {:#?}", err.to_string())
                }
            }
        }
    }
}
