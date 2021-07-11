use std::ops::AddAssign;

use crate::{Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath, Map};
use async_trait::async_trait;
use serde::Deserialize;

// average is (sum, count) pair
#[derive(Clone, Debug)]
pub struct Averagef32(pub f32, pub f32);

impl Averagef32 {
    pub fn new(sum: f32, count: f32) -> Self {
        Averagef32(sum, count)
    }

    pub fn average(&self) -> f32 {
        assert_ne!(self.1, 0.0, "divide by zero");
        self.0 / self.1
    }
}

impl AggregateAs<Averagef32> for u32 {
    fn aggregate_value(&self) -> Averagef32 {
        Averagef32(*self as f32, 1.0)
    }
}

impl AddAssign<Averagef32> for Averagef32 {
    fn add_assign(&mut self, rhs: Averagef32) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

#[derive(Deserialize)]
pub struct Averagef32AggregatorConfig {}

#[async_trait]
impl FromPath for Averagef32AggregatorConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(Averagef32AggregatorConfig {})
    }
}

impl ConfigInto<Averagef32Aggregator> for Averagef32AggregatorConfig {}

pub struct Averagef32Aggregator {}

#[async_trait]
impl FromConfig<Averagef32AggregatorConfig> for Averagef32Aggregator {
    async fn from_config(_config: &Averagef32AggregatorConfig) -> anyhow::Result<Self> {
        Ok(Averagef32Aggregator {})
    }
}

impl<I, T> Aggregate<I, T, Averagef32> for Averagef32Aggregator
where
    I: AggregateAs<Averagef32>,
    T: IntoIterator<Item = I>,
{
    fn merge(&self, u: &mut Averagef32, i: &I) {
        *u += i.aggregate_value()
    }
}

#[async_trait]
impl<I, T> Map<T, Averagef32, Averagef32AggregatorConfig> for Averagef32Aggregator
where
    I: AggregateAs<Averagef32>,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Averagef32> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod test_avg {

    use crate::*;

    #[derive(Clone, Debug, AggregateAs)]
    struct Record {
        id: String,
        #[agg(avgf32)]
        value: i32,
    }

    #[tokio::test]
    async fn test_averagef32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Averagef32, 1024);
        let mut pipe = mapper!("average");
        let pipe = run_pipe!(pipe, Averagef32AggregatorConfig, [tx1], rx0);
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record {
                    id: "a".to_owned(),
                    value: 1,
                },
                Record {
                    id: "a".to_owned(),
                    value: 2,
                },
                Record {
                    id: "a".to_owned(),
                    value: 3,
                },
            ]],
        );
        f0.await;
        join_pipes!([pipe]);
        let avg = rx1.recv().await.expect("not average received");
        assert_eq!(2.0, avg.average())
    }
}