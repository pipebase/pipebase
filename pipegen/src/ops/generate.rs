use crate::api::{App, Entity, EntityAccept, Object, Pipe, VisitEntity};
pub trait Generate<T> {
    fn new(indent: usize) -> Self;
    fn generate(&self) -> Option<String>;
}

pub struct PipeGenerator {
    indent: usize,
    pipe: Option<Pipe>,
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
    indent: usize,
    object: Option<Object>,
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

pub struct AppGenerator {
    indent: usize,
    app: Option<App>,
}

impl VisitEntity<App> for AppGenerator {
    fn visit(&mut self, app: &App) {
        self.app = Some(app.to_owned())
    }
}

impl Generate<App> for AppGenerator {
    fn new(indent: usize) -> Self {
        AppGenerator {
            indent: indent,
            app: None,
        }
    }

    fn generate(&self) -> Option<String> {
        self.generate_all()
    }
}

impl AppGenerator {
    pub fn generate_entity<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
        entity: &T,
        indent: usize,
    ) -> Option<String> {
        let mut generator = G::new(indent);
        entity.accept(&mut generator);
        generator.generate()
    }

    pub fn generate_entities<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
        entities: &Vec<T>,
        indent: usize,
        join_sep: &str,
    ) -> String {
        let mut lits: Vec<String> = vec![];
        for entity in entities.as_slice() {
            match Self::generate_entity(entity, indent) {
                Some(lit) => lits.push(lit),
                None => continue,
            }
        }
        lits.join(join_sep)
    }

    pub fn generate_objects(&self, indent: usize) -> Option<String> {
        let objects = match self.app {
            Some(ref app) => app.get_objects(),
            None => return None,
        };
        let objects = match objects {
            Some(objects) => objects,
            None => return None,
        };
        let objects_lit =
            Self::generate_entities::<Object, ObjectGenerator>(objects, indent, "\n\n");
        Some(objects_lit)
    }

    pub fn generate_pipes(&self, indent: usize) -> Option<String> {
        let pipes = match self.app {
            Some(ref app) => app.get_pipes(),
            None => return None,
        };
        let pipes_lit = Self::generate_entities::<Pipe, PipeGenerator>(pipes, indent, "\n");
        Some(pipes_lit)
    }

    pub fn generate_app_object(&self, indent: usize) -> Option<String> {
        match self.app {
            Some(ref app) => Some(app.to_literal(indent)),
            None => None,
        }
    }

    pub fn generate_all(&self) -> Option<String> {
        let module_name = match self.app {
            Some(ref app) => app.get_id(),
            None => return None,
        };
        let mut sections: Vec<String> = vec![];
        let indent: usize = self.indent + 1;
        match self.generate_objects(indent) {
            Some(objects_lit) => sections.push(objects_lit),
            None => (),
        };
        match self.generate_pipes(indent) {
            Some(pipes_lit) => sections.push(pipes_lit),
            None => (),
        };
        match self.generate_app_object(indent) {
            Some(app_object_lit) => sections.push(app_object_lit),
            None => (),
        };
        let module_lit = Self::generate_module(&module_name, &sections);
        Some(module_lit)
    }

    pub fn generate_module(module: &str, sections: &Vec<String>) -> String {
        format!("mod {} {{\n{}\n}}", module, sections.join("\n\n"))
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