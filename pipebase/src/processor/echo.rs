use std::fmt::Debug;

use super::Procedure;
use async_trait::async_trait;
use log::{info, warn};
pub struct Echo {}

#[async_trait]
impl<T: Debug + Send + Sync + 'static> Procedure<T, T> for Echo {
    async fn process(&self, t: T) -> T {
        info!("{:#?}", t);
        t
    }
}

#[cfg(test)]
mod tests {
    use super::Echo;
    use crate::processor::{Procedure, Processor};
    use std::println as info;
    use std::sync::mpsc::{channel, Sender};

    #[derive(Debug)]
    struct Message {
        m0: char,
        m1: i32,
    }

    async fn populate_message(tx0: Sender<Message>, message: Message) {
        tokio::spawn(async move { tx0.send(message).unwrap() }).await;
    }

    #[tokio::test]
    async fn test_echo() {
        let (tx0, rx0) = channel::<Message>();
        let (tx1, rx1) = channel::<Message>();
        let p = Processor { name: "echo" };
        let f0 = p.start::<Message, Message>(rx0, tx1, Box::new(Echo {}));
        let f1 = populate_message(tx0, Message { m0: 'a', m1: 1 });
        f1.await;
        f0.await;
        let message = rx1.recv().unwrap();
        assert_eq!('a', message.m0);
        assert_eq!(1, message.m1);
    }
}
