use crate::api::{Entity, EntityAccept, Object, Pipe, VisitEntity};
pub trait Generate<T> {
    fn do_generate(t: &T, indent: usize) -> Option<String>;
}

pub struct PipeGenerator {
    pub indent: usize,
    pub pipe: Option<Pipe>,
}

impl VisitEntity<Pipe> for PipeGenerator {
    fn visit(&mut self, pipe: &Pipe) {
        self.pipe = Some(pipe.to_owned())
    }
}

impl PipeGenerator {
    fn generate(&self) -> Option<String> {
        let pipe = match self.pipe.to_owned() {
            Some(p) => p,
            None => return None,
        };
        Some(pipe.to_literal(self.indent))
    }
}

impl Generate<Pipe> for PipeGenerator {
    fn do_generate(pipe: &Pipe, indent: usize) -> Option<String> {
        let mut pipe_generator = PipeGenerator {
            indent: indent,
            pipe: None,
        };
        pipe.accept(&mut pipe_generator);
        pipe_generator.generate()
    }
}

pub struct ObjectGenerator {
    pub indent: usize,
    pub object: Option<Object>,
}

impl VisitEntity<Object> for ObjectGenerator {
    fn visit(&mut self, object: &Object) {
        self.object = Some(object.to_owned())
    }
}

impl ObjectGenerator {
    fn generate(&self) -> Option<String> {
        let object = match self.object.to_owned() {
            Some(o) => o,
            None => return None,
        };
        Some(object.to_literal(self.indent))
    }
}

impl Generate<Object> for ObjectGenerator {
    fn do_generate(object: &Object, indent: usize) -> Option<String> {
        let mut generator = ObjectGenerator {
            indent: indent,
            object: None,
        };
        object.accept(&mut generator);
        generator.generate()
    }
}
