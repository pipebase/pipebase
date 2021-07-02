use crate::{Aggregate, AggregateAs, ConfigInto, FromConfig, FromPath, Map};
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

pub struct TopAggregator {
    n: usize,
    desc: bool,
}

impl TopAggregator {
    fn qsort<U>(v: &mut [U], n: usize, desc: bool)
    where
        U: Ord + Clone,
    {
        let n = n.min(v.len());
        let mut l = 0;
        let mut r = v.len() - 1;
        while l < r {
            let p = Self::partition(v, l, r, desc);
            if p + 1 == n {
                return;
            }
            if p + 1 > n {
                r = p - 1;
                continue;
            }
            l = p + 1
        }
    }

    fn partition<U>(v: &mut [U], l: usize, mut r: usize, desc: bool) -> usize
    where
        U: Ord + Clone,
    {
        let pivot = v[l].to_owned();
        let mut j = l;
        let mut i = l + 1;
        while i <= r {
            let u = v[i].to_owned();
            if (u >= pivot && desc) || (u <= pivot && !desc) {
                j += 1;
                v.swap(i, j);
                i += 1;
                continue;
            }
            v.swap(i, r);
            r -= 1;
        }
        v.swap(l, j);
        j
    }
}

#[async_trait]
impl FromConfig<TopAggregatorConfig> for TopAggregator {
    async fn from_config(config: &TopAggregatorConfig) -> anyhow::Result<Self> {
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
    fn aggregate(&self, t: T) -> Vec<U> {
        let mut buffer: Vec<U> = Vec::new();
        for item in t {
            buffer.extend(item.aggregate_value())
        }
        // apply n and desc
        Self::qsort(&mut buffer, self.n, self.desc);
        buffer.truncate(self.n.min(buffer.len()));
        buffer.sort_by(|a, b| match self.desc {
            true => b.partial_cmp(a).unwrap(),
            false => a.partial_cmp(b).unwrap(),
        });
        buffer
    }
}

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

    use crate::*;

    #[tokio::test]
    async fn test_top_aggregator() {
        let (tx0, rx0) = channel!(Vec<u32>, 1023);
        let (tx1, mut rx1) = channel!(Vec<u32>, 1024);
        let mut pipe = mapper!("top");
        let f0 = populate_records(
            tx0,
            vec![vec![1, 2, 2, 3], vec![1, 1, 2, 1], vec![2, 2, 2, 2]],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            TopAggregatorConfig,
            "resources/catalogs/top_aggregator.yml",
            [tx1],
            rx0
        )]);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![3, 2, 2], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 1, 1], r);
        let r = rx1.recv().await.unwrap();
        assert_eq!(vec![2, 2, 2], r);
    }
}
