use crate::{ConfigInto, FromConfig, FromPath, Project};

use super::Map;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct ProjectionConfig {}

#[async_trait]
impl FromPath for ProjectionConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
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

    #[tokio::test]
    async fn test_reverse_processor() {
        let (tx0, rx0) = channel!(Record, 1024);
        let (tx1, mut rx1) = channel!(self::ReversedRecord, 1024);
        let mut pipe = mapper!("reversed");
        let context = pipe.get_context();
        let f1 = populate_records(tx0, vec![Record { r0: 0, r1: 1 }]);
        f1.await;
        join_pipes!([run_pipe!(pipe, ProjectionConfig, [tx1], rx0)]);
        let reversed_record = rx1.recv().await.unwrap();
        assert_eq!(1, reversed_record.r0);
        assert_eq!(0, reversed_record.r1);
        context.validate(State::Done, 1);
    }
}
