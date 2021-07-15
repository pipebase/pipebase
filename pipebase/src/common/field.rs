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

pub trait FieldAccept<F: Clone> {
    fn accept(&self, visitor: &mut FieldVisitor<F>);
}
