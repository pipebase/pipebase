use crate::models::{
    App, ContextStore, Entity, EntityAccept, ErrorHandler, Object, Pipe, VisitEntity,
};
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
        PipeGenerator { indent, pipe: None }
    }

    fn generate(&self) -> String {
        self.pipe
            .as_ref()
            .expect("pipe not inited")
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
            indent,
            object: None,
        }
    }

    fn generate(&self) -> String {
        self.object
            .as_ref()
            .expect("object not inited")
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
            indent,
            cstore: None,
        }
    }

    fn generate(&self) -> String {
        self.cstore
            .as_ref()
            .expect("cstore not inited")
            .to_literal(self.indent)
    }
}

pub struct ErrorHandlerGenerator {
    indent: usize,
    error_handler: Option<ErrorHandler>,
}

impl VisitEntity<ErrorHandler> for ErrorHandlerGenerator {
    fn visit(&mut self, error_handler: &ErrorHandler) {
        self.error_handler = Some(error_handler.to_owned())
    }
}

impl Generate for ErrorHandlerGenerator {
    fn new(indent: usize) -> Self {
        ErrorHandlerGenerator {
            indent,
            error_handler: None,
        }
    }

    fn generate(&self) -> String {
        self.error_handler
            .as_ref()
            .expect("error handler not inited")
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
            indent,
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
        entity.accept_entity_visitor(&mut generator);
        generator.generate()
    }

    fn generate_entities<T: EntityAccept<G>, G: Generate + VisitEntity<T>>(
        entities: &[T],
        indent: usize,
        join_sep: &str,
    ) -> String {
        let lits: Vec<String> = entities
            .iter()
            .map(|entity| Self::generate_entity(entity, indent))
            .collect();
        lits.join(join_sep)
    }

    fn get_app(&self) -> &App {
        self.app.as_ref().expect("app inited")
    }

    fn generate_objects(&self, indent: usize) -> String {
        let objects = self.get_app().get_objects();
        Self::generate_entities::<Object, ObjectGenerator>(objects, indent, "\n\n")
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
        Self::generate_entities::<Pipe, PipeGenerator>(&pipes, indent, "\n")
    }

    fn generate_context_store(&self, indent: usize) -> String {
        let cstores = self.get_app().get_context_stores();
        Self::generate_entities::<ContextStore, ContextStoreGenerator>(cstores, indent, "\n")
    }

    fn generate_error_handler(&self, indent: usize) -> String {
        let error_handler = self.get_app().get_error_handler();
        let error_handler = match error_handler {
            Some(error_handler) => error_handler,
            None => return String::new(),
        };
        Self::generate_entity::<ErrorHandler, ErrorHandlerGenerator>(error_handler, indent)
    }

    fn generate_app_object(&self, indent: usize) -> String {
        self.get_app().to_literal(indent)
    }

    fn generate_app_metas(&self, indent: usize) -> String {
        self.get_app().get_app_metas_lit(indent)
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
        sections.push(self.generate_app_metas(indent));
        sections.push(self.generate_pipes(indent));
        sections.push(self.generate_context_store(indent));
        sections.push(self.generate_error_handler(indent));
        sections.push(self.generate_app_object(indent));
        sections.push(self.generate_bootstrap_app_function(indent));
        let module_lit = Self::generate_module(&module_name, &sections);
        let main_function_lit = self.generate_main_function(self.indent);
        format!("{}\n\n{}", module_lit, main_function_lit)
    }

    fn generate_module(module: &str, sections: &[String]) -> String {
        let sections: Vec<String> = sections.iter().cloned().filter(|s| !s.is_empty()).collect();
        format!("mod {} {{\n{}\n}}", module, sections.join("\n\n"))
    }

    pub fn set_pipe_filter(&mut self, selected_pipes: HashSet<String>) {
        self.pipe_filter = Some(selected_pipes);
    }
}

#[cfg(test)]
mod tests {
    use crate::models::App;
    use std::path::Path;

    #[test]
    fn test_complex_object_pipe() {
        let manifest_path = Path::new("resources/manifest/complex_object_pipe.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_print_timer_tick_pipe() {
        let manifest_path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_projection_pipe() {
        let manifest_path = Path::new("resources/manifest/projection_pipe.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_object_metas() {
        let manifest_path = Path::new("resources/manifest/object_metas.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }
}
