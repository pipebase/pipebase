use super::Map;
use crate::common::{Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TopAggregatorConfig {
    pub n: usize,
    pub desc: bool,
}

impl FromPath for TopAggregatorConfig {}

#[async_trait]
impl ConfigInto<TopAggregator> for TopAggregatorConfig {}

/// Find topN
pub struct TopAggregator {
    n: usize,
    desc: bool,
}

impl TopAggregator {
    fn qsort<U>(array: &mut [U], n: usize, desc: bool)
    where
        U: Ord + Clone,
    {
        let n = n.min(array.len());
        let mut left = 0;
        let mut right = array.len() - 1;
        while left < right {
            let position = Self::partition(array, left, right, desc);
            if position + 1 == n {
                return;
            }
            if position + 1 > n {
                right = position - 1;
                continue;
            }
            left = position + 1
        }
    }

    fn partition<U>(v: &mut [U], left: usize, mut right: usize, desc: bool) -> usize
    where
        U: Ord + Clone,
    {
        let pivot = v[left].to_owned();
        let mut j = left;
        let mut i = left + 1;
        while i <= right {
            let u = v[i].to_owned();
            if (u >= pivot && desc) || (u <= pivot && !desc) {
                j += 1;
                v.swap(i, j);
                i += 1;
                continue;
            }
            v.swap(i, right);
            right -= 1;
        }
        v.swap(left, j);
        j
    }
}

#[async_trait]
impl FromConfig<TopAggregatorConfig> for TopAggregator {
    async fn from_config(config: TopAggregatorConfig) -> anyhow::Result<Self> {
        Ok(TopAggregator {
            n: config.n,
            desc: config.desc,
        })
    }
}

impl<I, T, U> Aggregate<I, T, Vec<U>> for TopAggregator
where
    I: AggregateAs<Vec<U>>,
    U: Ord + Clone,
    T: IntoIterator<Item = I>,
{
    /// Merge comparable items into vectors
    fn merge(&self, u: &mut Vec<U>, i: &I) {
        u.extend(i.aggregate_value());
    }

    /// Sort merged items
    fn operate(&self, u: &mut Vec<U>) {
        Self::qsort(u, self.n, self.desc);
        u.truncate(self.n);
        u.sort_by(|a, b| match self.desc {
            true => b.partial_cmp(a).unwrap(),
            false => a.partial_cmp(b).unwrap(),
        });
    }
}

/// # Parameters
/// * T: input
/// * Vec<U>: output
#[async_trait]
impl<I, T, U> Map<T, Vec<U>, TopAggregatorConfig> for TopAggregator
where
    I: AggregateAs<Vec<U>>,
    U: Ord + Clone,
    T: IntoIterator<Item = I> + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Vec<U>> {
        Ok(self.aggregate(data))
    }
}

#[cfg(test)]
mod top_aggregator_tests {

    use crate::prelude::*;

    #[tokio::test]
    async fn test_top_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(Vec<u32>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(
            TopAggregatorConfig,
            "resources/catalogs/top_aggregator_desc.yml"
        );
        let pipe = mapper!("top");
        let f0 = populate_records(
            tx0,
            vec![vec![1, 2, 2, 3], vec![1, 1, 2, 1], vec![2, 2, 2, 2]],
        );
        f0.await;
        join_pipes!([run_pipe!(pipe, config, channels)]);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![3, 2, 2], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 1, 1], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 2, 2], r);
    }

    #[derive(AggregateAs, Clone, Debug, Equal, Eq, OrderedBy)]
    #[agg(top)]
    struct Record {
        pub id: String,
        #[equal]
        #[order]
        pub value: u32,
    }

    impl Record {
        pub fn new(id: &str, value: u32) -> Self {
            Record {
                id: id.to_owned(),
                value: value,
            }
        }
    }

    #[tokio::test]
    async fn test_top_record() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Record>, 1024);
        let channels = pipe_channels!(rx0, [tx1]);
        let config = config!(
            TopAggregatorConfig,
            "resources/catalogs/top_aggregator_asc.yml"
        );
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record::new("five", 5),
                Record::new("four", 4),
                Record::new("two", 2),
                Record::new("one", 1),
                Record::new("three", 3),
            ]],
        );
        f0.await;
        let pipe = mapper!("top_record");
        let pipe = run_pipe!(pipe, config, channels);
        let _ = pipe.await;
        let mut sorted_records = rx1.recv().await.unwrap();
        assert_eq!(3, sorted_records.len());
        let record = sorted_records.pop().unwrap();
        assert_eq!("three", &record.id);
        assert_eq!(3, record.value);
        let record = sorted_records.pop().unwrap();
        assert_eq!("two", &record.id);
        assert_eq!(2, record.value);
        let record = sorted_records.pop().unwrap();
        assert_eq!("one", &record.id);
        assert_eq!(1, record.value);
    }
}
