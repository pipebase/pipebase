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

/// Project from type to type
pub struct Projection {}

#[async_trait]
impl FromConfig<ProjectionConfig> for Projection {
    async fn from_config(_config: &ProjectionConfig) -> anyhow::Result<Self> {
        Ok(Projection {})
    }
}

/// # Parameters
/// * T: Input
/// * U: Output
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
    struct SwappedRecord {
        #[project(from = "r1")]
        pub r0: i32,
        #[project(from = "r0")]
        pub r1: i32,
    }

    #[tokio::test]
    async fn test_reverse_processor() {
        let (tx0, rx0) = channel!(Record, 1024);
        let (tx1, mut rx1) = channel!(self::SwappedRecord, 1024);
        let mut pipe = mapper!("swapped");
        let context = pipe.get_context();
        let f1 = populate_records(tx0, vec![Record { r0: 0, r1: 1 }]);
        f1.await;
        join_pipes!([run_pipe!(pipe, ProjectionConfig, [tx1], rx0)]);
        let swapped_record = rx1.recv().await.unwrap();
        assert_eq!(1, swapped_record.r0);
        assert_eq!(0, swapped_record.r1);
        context.validate(State::Done, 1);
    }
}
