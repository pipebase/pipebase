use crate::api::{Entity, EntityAccept, Object, Pipe, VisitEntity};
pub trait Generate<T> {
    fn generate(&self) -> Option<String>;
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

impl Generate<Pipe> for PipeGenerator {
    fn generate(&self) -> Option<String> {
        let pipe = match self.pipe.to_owned() {
            Some(p) => p,
            None => return None,
        };
        Some(pipe.to_literal(self.indent))
    }

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

impl Generate<Object> for ObjectGenerator {
    fn generate(&self) -> Option<String> {
        let object = match self.object.to_owned() {
            Some(o) => o,
            None => return None,
        };
        Some(object.to_literal(self.indent))
    }

    fn do_generate(object: &Object, indent: usize) -> Option<String> {
        let mut generator = ObjectGenerator {
            indent: indent,
            object: None,
        };
        object.accept(&mut generator);
        generator.generate()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::App;

    #[test]
    fn test_complex_object_pipe() {
        let manifest_path = "resources/manifest/complex_object_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_print_timer_tick_pipe() {
        let manifest_path = "resources/manifest/print_timer_tick_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_projection_pipe() {
        let manifest_path = "resources/manifest/projection_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }
}
