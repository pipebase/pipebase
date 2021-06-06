mod echo;
mod field;
mod filter;
mod project;

use std::fmt::Debug;

use crate::error::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};
#[async_trait]
pub trait Procedure<T, U>: Send + Sync {
    async fn process(&self, data: T) -> Result<U>;
}

pub struct Process<'a, T, U> {
    name: &'a str,
    rx: Receiver<T>,
    txs: Vec<Sender<U>>,
    p: Box<dyn Procedure<T, U>>,
}

impl<'a, T, U: Clone + Debug> Process<'a, T, U> {
    pub async fn run(&mut self) {
        loop {
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            let u = match self.p.process(t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    continue;
                }
            };
            for tx in self.txs.as_mut_slice() {
                match tx.send(u.to_owned()).await {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("processer send error {:#?}", err);
                    }
                }
            }
        }
    }
}
