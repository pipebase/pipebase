/// Field visitor visits object's field
pub struct FieldVisitor<F: Clone> {
    value: Option<F>,
}

impl<F: Clone> FieldVisitor<F> {
    pub fn new() -> Self {
        FieldVisitor { value: None }
    }

    pub fn visit(&mut self, value: &F) {
        self.value = Some(value.to_owned());
    }

    pub fn get_value(&self) -> Option<&F> {
        self.value.as_ref()
    }
}

/// Accept field visitor
pub trait FieldAccept<F: Clone> {
    fn accept(&self, visitor: &mut FieldVisitor<F>);
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
