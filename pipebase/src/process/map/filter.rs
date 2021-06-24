use std::iter::{FromIterator, IntoIterator};

use crate::{ConfigInto, FromConfig, FromFile};

use super::Map;
use async_trait::async_trait;
use serde::Deserialize;

pub trait Filter<Rhs = Self>: Clone {
    fn filter(rhs: &Rhs) -> Option<Rhs>;
}

#[derive(Deserialize)]
pub struct FilterMapConfig {}

impl FromFile for FilterMapConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(FilterMapConfig {})
    }
}

#[async_trait]
impl ConfigInto<FilterMap> for FilterMapConfig {}

pub struct FilterMap {}

#[async_trait]
impl FromConfig<FilterMapConfig> for FilterMap {
    async fn from_config(
        _config: &FilterMapConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(FilterMap {})
    }
}

#[async_trait]
impl<
        T: Filter + Clone + Sync,
        U: IntoIterator<Item = T> + Sync + Clone,
        V: FromIterator<T> + Send,
    > Map<U, V, FilterMapConfig> for FilterMap
{
    async fn map(&mut self, data: &U) -> std::result::Result<V, Box<dyn std::error::Error>> {
        Ok(data
            .to_owned()
            .into_iter()
            .filter_map(|item| T::filter(&item))
            .collect::<V>())
    }
}

#[cfg(test)]
mod tests {
    use crate::{channel, mapper, spawn_join, FilterMapConfig, FromFile, Mapper, Pipe};
    use pipederive::Filter;

    use super::Filter;

    #[derive(Clone, Debug, Filter)]
    #[filter(alias = "r", predicate = "r.r0 + r.r1 < 1")]
    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[test]
    fn test_filter() {
        let mut r = Record { r0: 1, r1: 1 };
        let maybe: Option<Record> = Record::filter(&r);
        match maybe {
            Some(_) => panic!("expect None"),
            None => (),
        }
        r.r0 = 0;
        r.r1 = 0;
        let maybe: Option<Record> = Record::filter(&r);
        match maybe {
            Some(_) => (),
            None => panic!("expect Some"),
        }
    }

    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: &mut Sender<Vec<Record>>, records: Vec<Record>) {
        let _ = tx.send(records).await;
    }
    #[tokio::test]
    async fn test_filter_map() {
        let (mut tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<self::Record>, 1024);
        let mut pipe = mapper!("filter_map", FilterMapConfig, rx0, [tx1]);
        let f1 = populate_records(
            &mut tx0,
            vec![
                Record { r0: 1, r1: 0 },
                Record { r0: 0, r1: 1 },
                Record { r0: 0, r1: 0 },
            ],
        );
        f1.await;
        drop(tx0);
        spawn_join!(pipe);
        let filtered_records = rx1.recv().await.unwrap();
        assert_eq!(1, filtered_records.len());
        assert_eq!(0, filtered_records[0].r0);
        assert_eq!(0, filtered_records[0].r1);
    }
}
