use crate::api::{App, ContextStore, Entity, EntityAccept, Object, Pipe, VisitEntity};
use std::collections::HashSet;
pub trait Generate {
    fn new(indent: usize) -> Self;
    fn generate(&self) -> String;
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

impl Generate for PipeGenerator {
    fn new(indent: usize) -> Self {
        PipeGenerator {
            indent: indent,
            pipe: None,
        }
    }

    fn generate(&self) -> String {
        self.pipe
            .as_ref()
            .expect("pipe inited")
            .to_literal(self.indent)
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

impl Generate for ObjectGenerator {
    fn new(indent: usize) -> Self {
        ObjectGenerator {
            indent: indent,
            object: None,
        }
    }

    fn generate(&self) -> String {
        self.object
            .as_ref()
            .expect("object inited")
            .to_literal(self.indent)
    }
}

pub struct ContextStoreGenerator {
    indent: usize,
    cstore: Option<ContextStore>,
}

impl VisitEntity<ContextStore> for ContextStoreGenerator {
    fn visit(&mut self, cstore: &ContextStore) {
        self.cstore = Some(cstore.to_owned())
    }
}

impl Generate for ContextStoreGenerator {
    fn new(indent: usize) -> Self {
        ContextStoreGenerator {
            indent: indent,
            cstore: None,
        }
    }

    fn generate(&self) -> String {
        self.cstore
            .as_ref()
            .expect("cstore inited")
            .to_literal(self.indent)
    }
}

pub struct AppGenerator {
    indent: usize,
    app: Option<App>,
    pipe_filter: Option<HashSet<String>>,
}

impl VisitEntity<App> for AppGenerator {
    fn visit(&mut self, app: &App) {
        self.app = Some(app.to_owned())
    }
}

impl Generate for AppGenerator {
    fn new(indent: usize) -> Self {
        AppGenerator {
            indent: indent,
            app: None,
            pipe_filter: None,
        }
    }

    fn generate(&self) -> String {
        self.generate_all()
    }
}

impl AppGenerator {
    fn generate_entity<T: EntityAccept<G>, G: Generate + VisitEntity<T>>(
        entity: &T,
        indent: usize,
    ) -> String {
        let mut generator = G::new(indent);
        entity.accept(&mut generator);
        generator.generate()
    }

    fn generate_entities<T: EntityAccept<G>, G: Generate + VisitEntity<T>>(
        entities: &Vec<T>,
        indent: usize,
        join_sep: &str,
    ) -> String {
        let lits: Vec<String> = entities
            .into_iter()
            .map(|entity| Self::generate_entity(entity, indent))
            .collect();
        lits.join(join_sep)
    }

    fn get_app(&self) -> &App {
        self.app.as_ref().expect("app inited")
    }

    fn generate_objects(&self, indent: usize) -> String {
        let objects = self.get_app().get_objects();
        let objects_lit =
            Self::generate_entities::<Object, ObjectGenerator>(objects, indent, "\n\n");
        objects_lit
    }

    fn generate_pipes(&self, indent: usize) -> String {
        let pipes = self.get_app().get_pipes().to_owned();
        let pipes = match self.pipe_filter {
            Some(ref filter) => pipes
                .into_iter()
                .filter(|pipe| filter.contains(&pipe.get_id()))
                .map(|mut pipe| {
                    pipe.filter_upstreams(filter);
                    pipe
                })
                .collect(),
            None => pipes,
        };
        let pipes_lit = Self::generate_entities::<Pipe, PipeGenerator>(&pipes, indent, "\n");
        pipes_lit
    }

    fn generate_context_store(&self, indent: usize) -> String {
        let cstores = self.get_app().get_context_stores();
        Self::generate_entities::<ContextStore, ContextStoreGenerator>(cstores, indent, "\n")
    }

    fn generate_app_object(&self, indent: usize) -> String {
        self.get_app().to_literal(indent)
    }

    fn generate_bootstrap_app_function(&self, indent: usize) -> String {
        self.get_app().get_bootstrap_function_literal(indent)
    }

    fn generate_main_function(&self, indent: usize) -> String {
        self.get_app().get_main_function_literal(indent)
    }

    // module paths
    fn generate_use_modules(&self, indent: usize) -> String {
        self.get_app().get_use_modules_literal(indent)
    }

    fn generate_all(&self) -> String {
        let module_name = self.get_app().get_app_module_name();
        let mut sections: Vec<String> = vec![];
        let indent: usize = self.indent + 1;
        sections.push(self.generate_use_modules(indent));
        sections.push(self.generate_objects(indent));
        sections.push(self.generate_pipes(indent));
        sections.push(self.generate_context_store(indent));
        sections.push(self.generate_app_object(indent));
        sections.push(self.generate_bootstrap_app_function(indent));
        let module_lit = Self::generate_module(&module_name, &sections);
        let main_function_lit = self.generate_main_function(self.indent);
        format!("{}\n\n{}", module_lit, main_function_lit)
    }

    fn generate_module(module: &str, sections: &Vec<String>) -> String {
        let sections: Vec<String> = sections
            .to_owned()
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        format!("mod {} {{\n{}\n}}", module, sections.join("\n\n"))
    }

    pub fn set_pipe_filter(&mut self, selected_pipes: HashSet<String>) {
        self.pipe_filter = Some(selected_pipes);
    }
}

#[cfg(test)]
mod tests {
    use crate::api::App;
    use std::path::Path;

    #[test]
    fn test_complex_object_pipe() {
        let manifest_path = Path::new("resources/manifest/complex_object_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_print_timer_tick_pipe() {
        let manifest_path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_projection_pipe() {
        let manifest_path = Path::new("resources/manifest/projection_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_object_metas() {
        let manifest_path = Path::new("resources/manifest/object_metas.yml");
        let app = App::read(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }
}
