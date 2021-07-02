use crate::{ConfigInto, FromConfig, FromPath};

use super::Map;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

pub trait Project<Rhs = Self> {
    fn project(rhs: &Rhs) -> Self;
}

impl Project<bool> for bool {
    fn project(from: &bool) -> bool {
        *from
    }
}

impl Project<i8> for i8 {
    fn project(from: &i8) -> i8 {
        *from
    }
}

impl Project<u8> for u8 {
    fn project(from: &u8) -> u8 {
        *from
    }
}

impl Project<i16> for i16 {
    fn project(from: &i16) -> i16 {
        *from
    }
}

impl Project<u16> for u16 {
    fn project(from: &u16) -> u16 {
        *from
    }
}

impl Project<i32> for i32 {
    fn project(from: &i32) -> i32 {
        *from
    }
}

impl Project<u32> for u32 {
    fn project(from: &u32) -> u32 {
        *from
    }
}

impl Project<isize> for isize {
    fn project(from: &isize) -> isize {
        *from
    }
}

impl Project<usize> for usize {
    fn project(from: &usize) -> usize {
        *from
    }
}

impl Project<i64> for i64 {
    fn project(from: &i64) -> i64 {
        *from
    }
}

impl Project<u64> for u64 {
    fn project(from: &u64) -> u64 {
        *from
    }
}

impl Project<i128> for i128 {
    fn project(from: &i128) -> i128 {
        *from
    }
}

impl Project<u128> for u128 {
    fn project(from: &u128) -> u128 {
        *from
    }
}

impl Project<f32> for f32 {
    fn project(from: &f32) -> f32 {
        *from
    }
}

impl Project<f64> for f64 {
    fn project(from: &f64) -> f64 {
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

/// Project Fixed Size Array
impl<T, U, const N: usize> Project<[T; N]> for [U; N]
where
    U: Project<T> + Default + Copy,
{
    fn project(from: &[T; N]) -> [U; N] {
        let mut to: [U; N] = [U::default(); N];
        for i in 0..N {
            to[i] = U::project(&from[i])
        }
        to
    }
}

/// Project Vec
impl<T, U> Project<Vec<T>> for Vec<U>
where
    U: Project<T>,
{
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

impl FromPath for ProjectionConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(ProjectionConfig {})
    }
}

#[async_trait]
impl ConfigInto<Projection> for ProjectionConfig {}

pub struct Projection {}

#[async_trait]
impl FromConfig<ProjectionConfig> for Projection {
    async fn from_config(_config: &ProjectionConfig) -> anyhow::Result<Self> {
        Ok(Projection {})
    }
}

#[async_trait]
impl<T, U> Map<T, U, ProjectionConfig> for Projection
where
    T: Send + 'static,
    U: Project<T>,
{
    async fn map(&mut self, data: T) -> anyhow::Result<U> {
        Ok(U::project(&data))
    }
}

#[cfg(test)]
mod tests {

    use crate::*;
    use tokio::sync::mpsc::Sender;

    #[derive(Debug)]
    struct Record {
        pub r0: i32,
        pub r1: i32,
    }

    #[derive(Clone, Debug, Project)]
    #[project(input = "self::Record")]
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
    #[project(input = "Record")]
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

    async fn populate_record(tx: Sender<Record>, r: Record) {
        tx.send(r).await.unwrap();
    }
    #[tokio::test]
    async fn test_reverse_processor() {
        let (tx0, rx0) = channel!(Record, 1024);
        let (tx1, mut rx1) = channel!(self::ReversedRecord, 1024);
        let mut pipe = mapper!("reversed");
        let context = pipe.get_context();
        let f1 = populate_record(tx0, Record { r0: 0, r1: 1 });
        f1.await;
        join_pipes!([run_pipe!(pipe, ProjectionConfig, [tx1], rx0)]);
        let reversed_record = rx1.recv().await.unwrap();
        assert_eq!(1, reversed_record.r0);
        assert_eq!(0, reversed_record.r1);
        let ctx = context.read().await;
        (*ctx).validate(State::Done, 2, 2);
    }
}
