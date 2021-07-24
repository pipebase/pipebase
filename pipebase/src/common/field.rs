/// Field visitor visits object's field
pub struct FieldVisitor<F> {
    value: Option<F>,
}

impl<F> FieldVisitor<F> {
    pub fn new() -> Self {
        FieldVisitor { value: None }
    }

    pub fn visit(&mut self, value: F) {
        self.value = Some(value);
    }

    pub fn get_value(self) -> Option<F> {
        self.value
    }
}

/// Accept field visitor
pub trait FieldAccept<F> {
    fn accept(self, visitor: &mut FieldVisitor<F>);
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(FieldAccept)]
    struct Records {
        #[visit]
        records: [i32; 3],
    }

    #[test]
    fn test_field_visit() {
        let record = [1, 2, 3];
        let records = Records { records: record };
        let mut visitor = FieldVisitor::<[i32; 3]>::new();
        records.accept(&mut visitor);
        let visitor_record = visitor.get_value().unwrap().to_owned();
        assert_eq!(record, visitor_record)
    }
}
