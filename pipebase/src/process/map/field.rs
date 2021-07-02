use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

pub struct FieldVisitor<F: Clone> {
    value: Option<F>,
}

impl<F: Clone> FieldVisitor<F> {
    pub fn visit(&mut self, value: &F) {
        self.value = Some(value.to_owned());
    }

    pub fn get_value(&self) -> Option<&F> {
        self.value.as_ref()
    }
}

pub trait FieldAccept<F: Clone> {
    fn accept(&self, visitor: &mut FieldVisitor<F>);
}

#[derive(Deserialize)]
pub struct FieldVisitConfig {}

impl FromPath for FieldVisitConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(FieldVisitConfig {})
    }
}

#[async_trait]
impl ConfigInto<FieldVisit> for FieldVisitConfig {}

pub struct FieldVisit {}

#[async_trait]
impl FromConfig<FieldVisitConfig> for FieldVisit {
    async fn from_config(_config: &FieldVisitConfig) -> anyhow::Result<Self> {
        Ok(FieldVisit {})
    }
}

#[async_trait]
impl<T, U> Map<T, U, FieldVisitConfig> for FieldVisit
where
    T: FieldAccept<U> + Send + 'static,
    U: Clone,
{
    async fn map(&mut self, t: T) -> anyhow::Result<U> {
        let mut visitor = FieldVisitor::<U> { value: None };
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

    #[test]
    fn test_field_visit() {
        let record = [1, 2, 3];
        let records = Records { records: record };
        let mut visitor = FieldVisitor::<[i32; 3]> { value: None };
        records.accept(&mut visitor);
        let visitor_record = visitor.get_value().unwrap().to_owned();
        assert_eq!(record, visitor_record)
    }

    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<Records>, records: Records) {
        let _ = tx.send(records).await;
    }

    #[tokio::test]
    async fn test_field_visit_procedure() {
        let (tx0, rx0) = channel!(Records, 1024);
        let (tx1, mut rx1) = channel!([i32; 3], 1024);
        let mut pipe = mapper!("field_visit");
        let f1 = populate_records(tx0, Records { records: [1, 2, 3] });
        f1.await;
        join_pipes!([run_pipe!(pipe, FieldVisitConfig, [tx1], rx0)]);
        let received_records = rx1.recv().await.unwrap();
        assert_eq!([1, 2, 3], received_records)
    }
}
