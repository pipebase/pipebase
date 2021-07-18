use std::{fmt::Debug, iter::FromIterator};

use crate::{ConfigInto, FromConfig, FromPath, Stream};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct IteratorReaderConfig {}

#[async_trait]
impl FromPath for IteratorReaderConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(IteratorReaderConfig {})
    }
}

#[async_trait]
impl<U> ConfigInto<IteratorReader<U>> for IteratorReaderConfig {}

/// Stream iterator
pub struct IteratorReader<U> {
    /// Sender to notify downstreams
    tx: Option<Sender<U>>,
}

#[async_trait]
impl<U> FromConfig<IteratorReaderConfig> for IteratorReader<U> {
    async fn from_config(_config: IteratorReaderConfig) -> anyhow::Result<Self> {
        Ok(IteratorReader { tx: None })
    }
}

/// # Parameters
/// * T: input
/// * U: output
#[async_trait]
impl<T, U> Stream<T, U, IteratorReaderConfig> for IteratorReader<U>
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

    #[tokio::test]
    async fn test_iterator_reader() {
        let (tx0, rx0) = channel!(HashMap<String, u32>, 1024);
        let (tx1, mut rx1) = channel!((String, u32), 1024);
        let mut pipe = streamer!("tuple_streamer");
        let mut record: HashMap<String, u32> = HashMap::new();
        record.insert("one".to_owned(), 1);
        record.insert("two".to_owned(), 2);
        let f0 = populate_records(tx0, vec![record]);
        f0.await;
        join_pipes!([run_pipe!(pipe, IteratorReaderConfig, [tx1], rx0)]);
        let mut records: HashMap<String, u32> = HashMap::new();
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        assert_eq!(&1, records.get("one").unwrap());
        assert_eq!(&2, records.get("two").unwrap())
    }
}
