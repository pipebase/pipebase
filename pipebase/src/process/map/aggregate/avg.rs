use std::ops::AddAssign;

use crate::AggregateAs;

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

    pub fn sum(&self) -> f32 {
        self.0
    }

    pub fn count(&self) -> f32 {
        self.1
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
        let pipe = run_pipe!(pipe, AddAggregatorConfig, [tx1], rx0);
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

#[cfg(test)]
mod unordered_group_avg_f32_tests {

    use crate::*;

    #[derive(Clone, Debug, AggregateAs, GroupAs)]
    struct Record {
        #[group]
        id: String,
        #[agg(avgf32)]
        value: i32,
    }

    #[tokio::test]
    async fn test_unordered_group_avg_f32() {
        let (tx0, rx0) = channel!(Vec<Record>, 1024);
        let (tx1, mut rx1) = channel!(Vec<Pair<String, Averagef32>>, 1024);
        let mut pipe = mapper!("group_avg_f32");
        let pipe = run_pipe!(pipe, UnorderedGroupAddAggregatorConfig, [tx1], rx0);
        let f0 = populate_records(
            tx0,
            vec![vec![
                Record {
                    id: "foo".to_owned(),
                    value: 1,
                },
                Record {
                    id: "foo".to_owned(),
                    value: 2,
                },
                Record {
                    id: "bar".to_owned(),
                    value: 2,
                },
                Record {
                    id: "bar".to_owned(),
                    value: 3,
                },
            ]],
        );
        f0.await;
        join_pipes!([pipe]);
        let group_avgs = rx1.recv().await.expect("group average not found");
        for avg in group_avgs {
            match &avg.left()[..] {
                "foo" => {
                    assert_eq!(1.5, avg.right().average())
                }
                "bar" => {
                    assert_eq!(2.5, avg.right().average())
                }
                _ => unreachable!("unexpected group {}", avg.left()),
            }
        }
    }
}
