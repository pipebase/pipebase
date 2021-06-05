use super::Procedure;
use async_trait::async_trait;
use std::fmt::Debug;

pub trait Project<Rhs = Self> {
    fn project(rhs: &Rhs) -> Self;
}

impl Project<i32> for i32 {
    fn project(from: &i32) -> i32 {
        *from
    }
}

impl Project<char> for char {
    fn project(from: &char) -> char {
        *from
    }
}

impl Project<String> for String {
    fn project(from: &String) -> String {
        from.clone()
    }
}

impl<T, U: Project<T>> Project<Option<T>> for Option<U> {
    fn project(from: &Option<T>) -> Option<U> {
        match from {
            Some(t) => Some(U::project(t)),
            None => None,
        }
    }
}

/// Project fixed-size array
impl<T, U: Project<T> + Default + Copy, const N: usize> Project<[T; N]> for [U; N] {
    fn project(from: &[T; N]) -> [U; N] {
        let mut to: [U; N] = [U::default(); N];
        for i in 0..N {
            to[i] = U::project(&from[i])
        }
        to
    }
}

/// Project vec
impl<T, U: Project<T>> Project<Vec<T>> for Vec<U> {
    fn project(from: &Vec<T>) -> Vec<U> {
        let mut to: Vec<U> = vec![];
        for item in from {
            to.push(U::project(item))
        }
        to
    }
}

pub struct Projection {}

#[async_trait]
impl<T: Send + Sync + 'static, U: Project<T> + Send + Sync + 'static> Procedure<T, U>
    for Projection
{
    async fn process(&self, data: T) -> U {
        U::project(&data)
    }
}

#[cfg(test)]
mod tests {

    use crate::process::{
        project::{Project, Projection},
        Process,
    };
    use pipederive::Project;
    use std::sync::mpsc::{channel, Sender};

    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[derive(Debug, Project)]
    #[input(module = "self", schema = "Record")]
    struct ReversedRecord {
        #[project(from = "r1")]
        pub r0: i32,
        #[project(from = "r0")]
        pub r1: i32,
    }

    #[test]
    fn test_reverse() {
        let origin = Record { r0: 0, r1: 1 };
        let reversed: ReversedRecord = Project::project(&origin);
        assert_eq!(1, reversed.r0);
        assert_eq!(0, reversed.r1);
    }

    async fn populate_record(tx: Sender<Record>, r: Record) {
        tokio::spawn(async move {
            tx.send(r).unwrap();
        })
        .await;
    }
    #[tokio::test]
    async fn test_reverse_processor() {
        let (tx0, rx0) = channel::<Record>();
        let (tx1, rx1) = channel::<ReversedRecord>();
        let p = Process { name: "reverse" };
        let f0 = p.start::<Record, ReversedRecord>(rx0, tx1, Box::new(Projection {}));
        let f1 = populate_record(tx0, Record { r0: 0, r1: 1 });
        f1.await;
        match f0.await {
            Ok(()) => (),
            Err(e) => panic!("{:#?}", e),
        }
        let reversed_record = rx1.recv().unwrap();
        assert_eq!(1, reversed_record.r0);
        assert_eq!(0, reversed_record.r1);
    }
}
