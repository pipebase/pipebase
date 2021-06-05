use async_trait::async_trait;

use super::Procedure;

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
    async fn process(&self, t: T) -> U {
        let mut visitor = FieldVisitor::<U> { value: None };
        t.accept(&mut visitor);
        visitor.get_value().unwrap()
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

    use std::sync::mpsc::{channel, Sender};

    async fn populate_records(tx: Sender<Records>, records: Records) {
        tokio::spawn(async move { tx.send(records).unwrap() }).await;
    }

    #[tokio::test]
    async fn test_procedure() {
        let (tx0, rx0) = channel::<Records>();
        let (tx1, rx1) = channel::<[i32; 3]>();
        let p = Process {
            name: "field_visit",
        };
        let f0 = p.start(rx0, tx1, Box::new(FieldVisit {}));
        let f1 = populate_records(tx0, Records { records: [1, 2, 3] });
        f1.await;
        f0.await;
        let received_records = rx1.recv().unwrap();
        assert_eq!([1, 2, 3], received_records)
    }
}
