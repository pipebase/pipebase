use super::Procedure;
use async_trait::async_trait;
pub trait Filter<Rhs = Self>: Clone {
    fn filter(rhs: &Rhs) -> Option<Rhs>;
}

pub struct FilterMap {}

#[async_trait]
impl<T: Filter + Clone + Send + Sync + 'static> Procedure<Vec<T>, Vec<T>> for FilterMap {
    async fn process(&self, data: Vec<T>) -> Vec<T> {
        data.iter()
            .filter_map(|item| T::filter(item))
            .collect::<Vec<T>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::process::{filter::FilterMap, Process};
    use pipederive::Filter;

    use super::Filter;

    #[derive(Clone, Filter)]
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

    use std::sync::mpsc::{channel, Sender};

    async fn populate_records(tx: Sender<Vec<Record>>, records: Vec<Record>) {
        tokio::spawn(async move {
            tx.send(records).unwrap();
        })
        .await;
    }
    #[tokio::test]
    async fn test_flat_map() {
        let (tx0, rx0) = channel::<Vec<Record>>();
        let (tx1, rx1) = channel::<Vec<Record>>();
        let p = Process { name: "flat_map" };
        let f0 = p.start::<Vec<Record>, Vec<Record>>(rx0, tx1, Box::new(FilterMap {}));
        let f1 = populate_records(
            tx0,
            vec![
                Record { r0: 1, r1: 0 },
                Record { r0: 0, r1: 1 },
                Record { r0: 0, r1: 0 },
            ],
        );
        f1.await;
        f0.await;
        let filtered_records = rx1.recv().unwrap();
        assert_eq!(1, filtered_records.len());
        assert_eq!(0, filtered_records[0].r0);
        assert_eq!(0, filtered_records[0].r1);
    }
}
