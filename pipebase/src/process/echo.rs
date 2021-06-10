use std::fmt::Debug;

use crate::{FromConfig, FromFile};

use super::Procedure;
use async_trait::async_trait;
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EchoConfig {}

impl FromFile for EchoConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(EchoConfig {})
    }
}
pub struct Echo {}

#[async_trait]
impl FromConfig<EchoConfig> for Echo {
    async fn from_config(
        _config: &EchoConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Echo {})
    }
}

#[async_trait]
impl<T: Debug + Send + Sync + 'static> Procedure<T, T> for Echo {
    async fn process(&self, t: T) -> std::result::Result<T, Box<dyn std::error::Error>> {
        info!("{:#?}", t);
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::{Echo, EchoConfig, FromConfig, FromFile};
    use crate::process::Process;
    use crate::{channel, process, spawn_join};
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
        let (mut tx0, rx0) = channel!(Message, 1024);
        let (tx1, mut rx1) = channel!(Message, 1024);
        let mut p = process!("echo", "", EchoConfig, Echo, rx0, [tx1]);
        let f1 = populate_message(&mut tx0, Message { m0: 'a', m1: 1 });
        f1.await;
        drop(tx0);
        spawn_join!(p);
        let message = rx1.recv().await.unwrap();
        assert_eq!('a', message.m0);
        assert_eq!(1, message.m1);
    }
}
