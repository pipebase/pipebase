use crate::AggregateAs;
use std::{cmp::Ordering, fmt::Debug};

#[derive(Clone, Debug, Eq)]
pub struct Count32(pub u32);

impl Count32 {
    pub fn new(c: u32) -> Self {
        Count32(c)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl std::ops::AddAssign<Self> for Count32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Ord for Count32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Count32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl PartialEq for Count32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl AggregateAs<Count32> for Count32 {
    fn aggregate_value(&self) -> Count32 {
        self.to_owned()
    }
}

impl AggregateAs<Vec<Count32>> for Count32 {
    fn aggregate_value(&self) -> Vec<Count32> {
        vec![self.to_owned()]
    }
}

impl AggregateAs<Count32> for u32 {
    fn aggregate_value(&self) -> Count32 {
        Count32::new(1)
    }
}

impl AggregateAs<Count32> for String {
    fn aggregate_value(&self) -> Count32 {
        Count32::new(1)
    }
}

#[cfg(test)]
mod count32_tests {

    use crate::*;

    #[derive(Debug, Clone, AggregateAs)]
    #[agg(count32)]
    struct Record {}

    #[tokio::test]
    async fn test_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Count32, 1024);
        let mut pipe = mapper!("counter");
        let pipe = run_pipe!(pipe, AddAggregatorConfig, [tx1], rx0);
        let f0 = populate_records(tx0, vec![vec![Record {}, Record {}, Record {}, Record {}]]);
        f0.await;
        join_pipes!([pipe]);
        let c = rx1.recv().await.expect("count32 not found");
        assert_eq!(4, c.get())
    }
}

#[cfg(test)]
mod group_count32_tests {

    use crate::*;

    #[derive(Debug, Clone, GroupAs, AggregateAs)]
    #[agg(count32)]
    struct Record {
        #[group]
        key: String,
    }

    #[tokio::test]
    async fn test_word_group_count_aggregate() {
        let (tx0, rx0) = channel!(Vec<String>, 1024);
        let (tx1, mut rx2) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("word_count");
        let f0 = populate_records(
            tx0,
            vec![vec![
                "foo".to_owned(),
                "foo".to_owned(),
                "bar".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
                "buz".to_owned(),
            ]],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            UnorderedGroupAddAggregatorConfig,
            [tx1],
            rx0
        )]);
        let wcs = rx2.recv().await.unwrap();
        for wc in wcs {
            match wc.left().as_str() {
                "foo" => assert_eq!(2, wc.right().get()),
                "bar" => assert_eq!(1, wc.right().get()),
                "buz" => assert_eq!(3, wc.right().get()),
                _ => unreachable!(),
            }
        }
    }

    #[tokio::test]
    async fn test_record_group_count32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Count32>>, 1024);
        let mut pipe = mapper!("group_count32");
        let pipe = run_pipe!(pipe, UnorderedGroupAddAggregatorConfig, [tx1], rx0);
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record {
                    key: "foo".to_owned(),
                },
                Record {
                    key: "foo".to_owned(),
                },
                Record {
                    key: "bar".to_owned(),
                },
            ]],
        );
        f0.await;
        join_pipes!([pipe]);
        let group_counts = rx1.recv().await.expect("group count32 not found");
        for count in group_counts {
            match &count.left()[..] {
                "foo" => {
                    assert_eq!(2, count.right().get())
                }
                "bar" => {
                    assert_eq!(1, count.right().get())
                }
                _ => unreachable!("unexpected group {}", count.left()),
            }
        }
    }
}
