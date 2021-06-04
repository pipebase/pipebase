mod echo;

use crate::error::{join_error, Result};
use async_trait::async_trait;
use log::{error, info, warn};
use std::sync::mpsc::{Receiver, Sender};
#[async_trait]
pub trait Procedure<T, U>: Send + Sync {
    async fn process(&self, data: T) -> U;
}

pub struct Processor<'a> {
    name: &'a str,
}

impl<'a> Processor<'a> {
    pub async fn start<T: Send + Sync + 'static, U: Send + Sync + 'static>(
        &self,
        rx: Receiver<T>,
        tx: Sender<U>,
        p: Box<dyn Procedure<T, U>>,
    ) -> Result<()> {
        let join_handler = tokio::spawn(async move {
            for t in rx {
                let u: U = p.process(t).await;
                match tx.send(u) {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("processer send error {:#?}", err);
                    }
                }
            }
        });
        // TODO: Error Handling
        match join_handler.await {
            Ok(_) => Ok(()),
            Err(err) => Err(join_error(err)),
        }
    }
}
