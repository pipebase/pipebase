mod echo;
mod field;
mod filter;
mod project;

use std::fmt::Debug;

use async_trait::async_trait;
use log::error;
use std::error::Error;
use std::result::Result;
use tokio::sync::mpsc::{Receiver, Sender};
#[async_trait]
pub trait Procedure<T, U>: Send + Sync {
    async fn process(&self, data: T) -> Result<U, Box<dyn Error>>;
}

pub struct Process<'a, T, U> {
    name: &'a str,
    rx: Receiver<T>,
    tx: Sender<U>,
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

            match self.tx.send(u.to_owned()).await {
                Ok(_) => continue,
                Err(err) => {
                    error!("processer send error {:#?}", err);
                }
            }
        }
    }
}
