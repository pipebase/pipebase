use super::Procedure;
use async_trait::async_trait;
use std::error::Error;
use std::result::Result;
pub trait Filter<Rhs = Self>: Clone {
    fn filter(rhs: &Rhs) -> Option<Rhs>;
}

pub struct FilterMap {}

#[async_trait]
impl<T: Filter + Clone + Send + Sync + 'static> Procedure<Vec<T>, Vec<T>> for FilterMap {
    async fn process(&self, data: Vec<T>) -> Result<Vec<T>, Box<dyn Error>> {
        Ok(data
            .iter()
            .filter_map(|item| T::filter(item))
            .collect::<Vec<T>>())
    }
}

#[cfg(test)]
mod tests {
    use crate::process::{filter::FilterMap, Process};
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

    use tokio::sync::mpsc::{channel, Sender};

    async fn populate_records(tx: &mut Sender<Vec<Record>>, records: Vec<Record>) {
        tx.send(records).await;
    }
    #[tokio::test]
    async fn test_filter_map() {
        let (mut tx0, rx0) = channel::<Vec<Record>>(1024);
        let (tx1, mut rx1) = channel::<Vec<Record>>(1024);
        let mut p = Process {
            name: "filter_map",
            rx: rx0,
            tx: tx1,
            p: Box::new(FilterMap {}),
        };
        let f0 = p.run();
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
        f0.await;
        let filtered_records = rx1.recv().await.unwrap();
        assert_eq!(1, filtered_records.len());
        assert_eq!(0, filtered_records[0].r0);
        assert_eq!(0, filtered_records[0].r1);
    }
}
