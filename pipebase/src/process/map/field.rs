use super::Map;
use crate::{ConfigInto, FromConfig, FromFile};
use async_trait::async_trait;
use serde::Deserialize;

pub struct FieldVisitor<F: Clone> {
    value: Option<F>,
}

impl<F: Clone> FieldVisitor<F> {
    pub fn visit(&mut self, value: F) {
        self.value = Some(value);
    }

    pub fn get_value(&self) -> Option<F> {
        self.value.to_owned()
    }
}

pub trait FieldAccept<F: Clone> {
    fn accept(&self, visitor: &mut FieldVisitor<F>);
}

#[derive(Deserialize)]
pub struct FieldVisitConfig {}

impl FromFile for FieldVisitConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(FieldVisitConfig {})
    }
}

#[async_trait]
impl ConfigInto<FieldVisit> for FieldVisitConfig {}

pub struct FieldVisit {}
#[async_trait]
impl FromConfig<FieldVisitConfig> for FieldVisit {
    async fn from_config(
        _config: &FieldVisitConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(FieldVisit {})
    }
}

#[async_trait]
impl<T: FieldAccept<U> + Sync, U: Clone> Map<T, U, FieldVisitConfig> for FieldVisit {
    async fn map(&mut self, t: &T) -> std::result::Result<U, Box<dyn std::error::Error>> {
        let mut visitor = FieldVisitor::<U> { value: None };
        t.accept(&mut visitor);
        Ok(visitor.get_value().unwrap())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        channel, mapper, spawn_join, FieldAccept, FieldVisitConfig, FieldVisitor, FromFile, Mapper,
        Pipe,
    };

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
        let visitor_record = visitor.get_value().unwrap();
        assert_eq!(record, visitor_record)
    }

    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: &mut Sender<Records>, records: Records) {
        let _ = tx.send(records).await;
    }

    #[tokio::test]
    async fn test_field_visit_procedure() {
        let (mut tx0, rx0) = channel!(Records, 1024);
        let (tx1, mut rx1) = channel!([i32; 3], 1024);
        let mut pipe = mapper!("field_visit", FieldVisitConfig, rx0, [tx1]);
        let f1 = populate_records(&mut tx0, Records { records: [1, 2, 3] });
        f1.await;
        drop(tx0);
        spawn_join!(pipe);
        let received_records = rx1.recv().await.unwrap();
        assert_eq!([1, 2, 3], received_records)
    }
}
