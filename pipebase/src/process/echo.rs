use std::fmt::Debug;

use super::Procedure;
use async_trait::async_trait;
use log::info;
use std::error::Error;
use std::result::Result;
pub struct Echo {}

#[async_trait]
impl<T: Debug + Send + Sync + 'static> Procedure<T, T> for Echo {
    async fn process(&self, t: T) -> Result<T, Box<dyn Error>> {
        info!("{:#?}", t);
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::process::Process;
    use std::println as info;
    use tokio::sync::mpsc::{channel, Sender};

    #[derive(Clone, Debug)]
    struct Message {
        m0: char,
        m1: i32,
    }

    async fn populate_message(tx0: &mut Sender<Message>, message: Message) {
        tx0.send(message).await;
    }

    #[tokio::test]
    async fn test_echo() {
        let (mut tx0, rx0) = channel::<Message>(1024);
        let (tx1, mut rx1) = channel::<Message>(1024);
        let mut p = Process {
            name: "echo",
            rx: rx0,
            tx: Some(tx1),
            p: Box::new(Echo {}),
        };
        let f0 = p.run();
        let f1 = populate_message(&mut tx0, Message { m0: 'a', m1: 1 });
        f1.await;
        drop(tx0);
        f0.await;
        let message = rx1.recv().await.unwrap();
        assert_eq!('a', message.m0);
        assert_eq!(1, message.m1);
    }
}
