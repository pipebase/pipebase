mod echo;

use async_trait::async_trait;
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
    ) {
        let join_handler = tokio::spawn(async move {
            for t in rx {
                // TODO: Error Handling
                let u: U = p.process(t).await;
                tx.send(u);
            }
        });
        // TODO: Error Handling
        join_handler.await;
    }
}
