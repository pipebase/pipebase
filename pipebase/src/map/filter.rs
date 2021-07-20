use std::iter::{FromIterator, IntoIterator};

use crate::common::{ConfigInto, Filter, FromConfig, FromPath};

use super::Map;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct FilterMapConfig {}

#[async_trait]
impl FromPath for FilterMapConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(FilterMapConfig {})
    }
}

#[async_trait]
impl ConfigInto<FilterMap> for FilterMapConfig {}

/// Filter items in iterator
pub struct FilterMap {}

#[async_trait]
impl FromConfig<FilterMapConfig> for FilterMap {
    async fn from_config(_config: FilterMapConfig) -> anyhow::Result<Self> {
        Ok(FilterMap {})
    }
}

/// # Parameters
/// * U: input
/// * V: output
#[async_trait]
impl<T, U, V> Map<U, V, FilterMapConfig> for FilterMap
where
    T: Filter + Clone + Sync,
    U: IntoIterator<Item = T> + Send + Clone + 'static,
    V: FromIterator<T> + Send,
{
    async fn map(&mut self, data: U) -> anyhow::Result<V> {
        Ok(data
            .into_iter()
            .filter_map(|item| match T::filter(&item) {
                true => Some(item),
                false => None,
            })
            .collect::<V>())
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;

    #[derive(Clone, Debug, Filter)]
    #[filter(alias = "r", predicate = "r.r0 + r.r1 < 1")]
    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[tokio::test]
    async fn test_filter_map() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<self::Record>, 1024);
        let mut pipe = mapper!("filter_map");
        let f1 = populate_records(
            tx0,
            vec![vec![
                Record { r0: 1, r1: 0 },
                Record { r0: 0, r1: 1 },
                Record { r0: 0, r1: 0 },
            ]],
        );
        f1.await;
        join_pipes!([run_pipe!(pipe, FilterMapConfig, [tx1], rx0)]);
        let filtered_records = rx1.recv().await.unwrap();
        assert_eq!(1, filtered_records.len());
        assert_eq!(0, filtered_records[0].r0);
        assert_eq!(0, filtered_records[0].r1);
    }
}
