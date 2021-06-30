use std::{fmt::Debug, iter::FromIterator};

use crate::{ConfigInto, FromConfig, FromPath, Stream};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct IteratorStreamerConfig {}

impl FromPath for IteratorStreamerConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(IteratorStreamerConfig {})
    }
}

#[async_trait]
impl<U> ConfigInto<IteratorStreamer<U>> for IteratorStreamerConfig {}

pub struct IteratorStreamer<U> {
    tx: Option<Sender<U>>,
}

#[async_trait]
impl<U> FromConfig<IteratorStreamerConfig> for IteratorStreamer<U> {
    async fn from_config(_config: &IteratorStreamerConfig) -> anyhow::Result<Self> {
        Ok(IteratorStreamer { tx: None })
    }
}

#[async_trait]
impl<T, U> Stream<T, U, IteratorStreamerConfig> for IteratorStreamer<U>
where
    T: IntoIterator<Item = U> + Send + 'static,
    U: Debug + Send + Sync + 'static,
{
    async fn stream(&mut self, t: T) -> anyhow::Result<()> {
        let tx = self.tx.as_ref().unwrap();
        // Avoid the trait `Send` is not implemented for `< as IntoIterator>::IntoIter`
        let buffer: Vec<U> = Vec::from_iter(t);
        for item in buffer {
            tx.send(item).await?;
        }
        Ok(())
    }

    fn set_sender(&mut self, sender: Sender<U>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    use std::collections::HashMap;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<HashMap<String, u32>>, records: HashMap<String, u32>) {
        let _ = tx.send(records).await;
    }

    #[tokio::test]
    async fn test_streamer() {
        let (tx0, rx0) = channel!(HashMap<String, u32>, 1024);
        let (tx1, mut rx1) = channel!((String, u32), 1024);
        let mut pipe = streamer!("tuple_streamer", IteratorStreamerConfig, rx0, [tx1]);
        let mut records: HashMap<String, u32> = HashMap::new();
        records.insert("one".to_owned(), 1);
        records.insert("two".to_owned(), 2);
        let f0 = populate_records(tx0, records);
        f0.await;
        spawn_join!(pipe);
        let mut records: HashMap<String, u32> = HashMap::new();
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        assert_eq!(&1, records.get("one").unwrap());
        assert_eq!(&2, records.get("two").unwrap())
    }
}
