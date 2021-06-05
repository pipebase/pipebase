mod echo;
mod field;
mod filter;
mod project;

use crate::error::{join_error, Result};
use async_trait::async_trait;
use log::{error, info, warn};
use std::sync::mpsc::{Receiver, Sender};
#[async_trait]
pub trait Procedure<T, U>: Send + Sync {
    async fn process(&self, data: T) -> U;
}

pub struct Process<'a> {
    name: &'a str,
}

impl<'a> Process<'a> {
    pub async fn run<T: Send + Sync + 'static, U: Clone + Send + Sync + 'static>(
        &self,
        rx: Receiver<T>,
        txs: Vec<Sender<U>>,
        p: Box<dyn Procedure<T, U>>,
    ) -> Result<()> {
        let join_handler = tokio::spawn(async move {
            for t in rx {
                let u: U = p.process(t).await;
                for tx in txs.as_slice() {
                    match tx.send(u.to_owned()) {
                        Ok(_) => continue,
                        Err(err) => {
                            error!("processer send error {:#?}", err);
                        }
                    }
                }
            }
        });
        match join_handler.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }
}
