use super::Map;
use crate::{ConfigInto, FieldAccept, FieldVisitor, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct FieldVisitConfig {}

#[async_trait]
impl FromPath for FieldVisitConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(FieldVisitConfig {})
    }
}

#[async_trait]
impl ConfigInto<FieldVisit> for FieldVisitConfig {}

/// Visit Object Field
pub struct FieldVisit {}

#[async_trait]
impl FromConfig<FieldVisitConfig> for FieldVisit {
    async fn from_config(_config: &FieldVisitConfig) -> anyhow::Result<Self> {
        Ok(FieldVisit {})
    }
}

/// # Parameters
/// * T: Input
/// * U, field of T: Output
#[async_trait]
impl<T, U> Map<T, U, FieldVisitConfig> for FieldVisit
where
    T: FieldAccept<U> + Send + 'static,
    U: Clone,
{
    async fn map(&mut self, t: T) -> anyhow::Result<U> {
        let mut visitor = FieldVisitor::new();
        t.accept(&mut visitor);
        Ok(visitor.get_value().unwrap().to_owned())
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[derive(FieldAccept)]
    struct Records {
        #[visit]
        records: [i32; 3],
    }

    #[tokio::test]
    async fn test_field_visit_procedure() {
        let (tx0, rx0) = channel!(Records, 1024);
        let (tx1, mut rx1) = channel!([i32; 3], 1024);
        let mut pipe = mapper!("field_visit");
        let f1 = populate_records(tx0, vec![Records { records: [1, 2, 3] }]);
        f1.await;
        join_pipes!([run_pipe!(pipe, FieldVisitConfig, [tx1], rx0)]);
        let received_records = rx1.recv().await.unwrap();
        assert_eq!([1, 2, 3], received_records)
    }
}
