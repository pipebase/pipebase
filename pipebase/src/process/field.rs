use super::Procedure;
use async_trait::async_trait;
use std::error::Error;
use std::result::Result;

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

pub struct FieldVisit {}

#[async_trait]
impl<T: FieldAccept<U> + Send + Sync + 'static, U: Clone + Send + Sync + 'static> Procedure<T, U>
    for FieldVisit
{
    async fn process(&self, t: T) -> Result<U, Box<dyn Error>> {
        let mut visitor = FieldVisitor::<U> { value: None };
        t.accept(&mut visitor);
        Ok(visitor.get_value().unwrap())
    }
}

#[cfg(test)]
mod tests {

    use super::super::Process;
    use super::{FieldAccept, FieldVisit, FieldVisitor};
    use pipederive::FieldAccept;

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

    use tokio::sync::mpsc::{channel, Sender};

    async fn populate_records(tx: &mut Sender<Records>, records: Records) {
        tx.send(records).await;
    }

    #[tokio::test]
    async fn test_procedure() {
        let (mut tx0, rx0) = channel::<Records>(1024);
        let (tx1, mut rx1) = channel::<[i32; 3]>(1024);
        let mut p = Process {
            name: "field_visit",
            rx: rx0,
            txs: vec![tx1],
            p: Box::new(FieldVisit {}),
        };
        let f0 = p.run();
        let f1 = populate_records(&mut tx0, Records { records: [1, 2, 3] });
        f1.await;
        drop(tx0);
        f0.await;
        let received_records = rx1.recv().await.unwrap();
        assert_eq!([1, 2, 3], received_records)
    }
}
