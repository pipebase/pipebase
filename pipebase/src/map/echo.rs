use std::fmt::Debug;

use super::Map;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

#[derive(Deserialize)]
pub struct EchoConfig {}

#[async_trait]
impl FromPath for EchoConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(EchoConfig {})
    }
}

#[async_trait]
impl ConfigInto<Echo> for EchoConfig {}

/// Log and return input
pub struct Echo {}

#[async_trait]
impl FromConfig<EchoConfig> for Echo {
    async fn from_config(_config: EchoConfig) -> anyhow::Result<Self> {
        Ok(Echo {})
    }
}

/// # Parameters
/// * T: input
/// * T: output
#[async_trait]
impl<T> Map<T, T, EchoConfig> for Echo
where
    T: Clone + Debug + Send + 'static,
{
    async fn map(&mut self, t: T) -> anyhow::Result<T> {
        info!("{:#?}", t);
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use tokio::sync::mpsc::Sender;

    #[derive(Clone, Debug)]
    struct Message {
        m0: char,
        m1: i32,
    }

    async fn populate_message(tx0: Sender<Message>, message: Message) {
        let _ = tx0.send(message).await;
    }

    #[tokio::test]
    async fn test_echo() {
        let (tx0, rx0) = channel!(Message, 1024);
        let (tx1, mut rx1) = channel!(Message, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(EchoConfig);
        let pipe = mapper!("echo");
        let f1 = populate_message(tx0, Message { m0: 'a', m1: 1 });
        f1.await;
        join_pipes!([run_pipe!(pipe, config, channels)]);
        let message = rx1.recv().await.unwrap();
        assert_eq!('a', message.m0);
        assert_eq!(1, message.m1);
    }
}
