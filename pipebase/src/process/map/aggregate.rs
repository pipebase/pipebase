use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
pub trait Aggregate<I, T, U>
where
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> U;
}

#[derive(Deserialize)]
pub struct SumAggregatorConfig {}

impl FromPath for SumAggregatorConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(SumAggregatorConfig {})
    }
}

#[async_trait]
impl ConfigInto<SumAggregator> for SumAggregatorConfig {}

pub struct SumAggregator {}

#[async_trait]
impl FromConfig<SumAggregatorConfig> for SumAggregator {
    async fn from_config(_config: &SumAggregatorConfig) -> anyhow::Result<Self> {
        Ok(SumAggregator {})
    }
}

impl<I, T> Aggregate<I, T, I> for SumAggregator
where
    I: std::ops::Add<Output = I> + Default,
    T: IntoIterator<Item = I>,
{
    fn aggregate(&self, t: T) -> I {
        let mut sum: I = Default::default();
        for item in t.into_iter() {
            sum = sum.add(item);
        }
        sum
    }
}

#[async_trait]
impl<I, T> Map<T, I, SumAggregatorConfig> for SumAggregator
where
    I: std::ops::Add<Output = I> + Default,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<I> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_record(tx: Sender<Vec<usize>>, records: Vec<Vec<usize>>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_sum_aggregator() {
        let (tx0, rx0) = channel!(Vec<usize>, 1023);
        let (tx1, mut rx1) = channel!(usize, 1024);
        let mut pipe = mapper!("summation", SumAggregatorConfig, rx0, [tx1]);
        let f0 = populate_record(tx0, vec![vec![1, 3, 5, 7], vec![2, 4, 6, 8]]);
        f0.await;
        spawn_join!(pipe);
        let odd = rx1.recv().await.unwrap();
        assert_eq!(16, odd);
        let even = rx1.recv().await.unwrap();
        assert_eq!(20, even);
    }
}
