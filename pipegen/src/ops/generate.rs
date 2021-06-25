use crate::api::{Entity, Object, Pipe, VisitEntity};
pub trait Generate<T> {
    fn new(indent: usize) -> Self;
    fn generate(&self) -> Option<String>;
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
    fn new(indent: usize) -> Self {
        PipeGenerator {
            indent: indent,
            pipe: None,
        }
    }

    fn generate(&self) -> Option<String> {
        let pipe = match self.pipe {
            Some(ref p) => p,
            None => return None,
        };
        Some(pipe.to_literal(self.indent))
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
    fn new(indent: usize) -> Self {
        ObjectGenerator {
            indent: indent,
            object: None,
        }
    }

    fn generate(&self) -> Option<String> {
        let object = match self.object {
            Some(ref o) => o,
            None => return None,
        };
        Some(object.to_literal(self.indent))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::App;
    use std::path::Path;

    #[test]
    fn test_complex_object_pipe() {
        let manifest_path = Path::new("resources/manifest/complex_object_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_print_timer_tick_pipe() {
        let manifest_path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_projection_pipe() {
        let manifest_path = Path::new("resources/manifest/projection_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }
}
