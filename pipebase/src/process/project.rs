use crate::{FromConfig, FromFile};

use super::Procedure;
use async_trait::async_trait;
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct ProjectionConfig {}

impl FromFile for ProjectionConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(ProjectionConfig {})
    }
}

pub struct Projection {}

#[async_trait]
impl FromConfig<ProjectionConfig> for Projection {
    async fn from_config(
        _config: &ProjectionConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Projection {})
    }
}

#[async_trait]
impl<T: Send + 'static, U: Project<T>> Procedure<T, U> for Projection {
    async fn process(&self, data: T) -> std::result::Result<U, Box<dyn std::error::Error>> {
        Ok(U::project(&data))
    }
}

#[cfg(test)]
mod tests {

    use crate::process::{
        project::{Project, Projection, ProjectionConfig},
        Process,
    };
    use crate::{channel, process, spawn_join, FromConfig, FromFile};
    use pipederive::Project;
    use tokio::sync::mpsc::{channel, Sender};

    #[derive(Debug)]
    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[derive(Clone, Debug, Project)]
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

    #[derive(Debug, Project)]
    #[input(module = "self", schema = "Record")]
    struct RecordSumPlusOne {
        #[project(alias = "r", expr = "let mut s = r.r0 + r.r1; s + 1")]
        pub s: i32,
    }

    #[test]
    fn test_sum_plus_one() {
        let origin = Record { r0: 1, r1: 1 };
        let sum = RecordSumPlusOne::project(&origin);
        assert_eq!(3, sum.s);
    }

    async fn populate_record(tx: &mut Sender<Record>, r: Record) {
        tx.send(r).await.unwrap();
    }
    #[tokio::test]
    async fn test_reverse_processor() {
        let (mut tx0, rx0) = channel!(Record, 1024);
        let (tx1, mut rx1) = channel!(self::ReversedRecord, 1024);
        let mut pipe = process!("reverse", "", ProjectionConfig, Projection, rx0, [tx1]);
        let f1 = populate_record(&mut tx0, Record { r0: 0, r1: 1 });
        f1.await;
        drop(tx0);
        spawn_join!(pipe);
        let reversed_record = rx1.recv().await.unwrap();
        assert_eq!(1, reversed_record.r0);
        assert_eq!(0, reversed_record.r1);
    }
}
