use crate::api::pipe::Pipe;
use crate::api::utils::indent_literal;
use crate::error::*;
use crate::operation::{Generate, ObjectGenerator, PipeGenerator};
use serde::Deserialize;

use super::Entity;
use super::Object;

#[derive(Deserialize, Debug)]
pub struct App {
    pub name: String,
    pub pipes: Vec<Pipe>,
}

impl App {
    pub fn parse(api_manifest_path: &str) -> Result<App> {
        let file = match std::fs::File::open(api_manifest_path) {
            Ok(file) => file,
            Err(err) => return Err(io_error(err)),
        };
        let app = match serde_yaml::from_reader::<std::fs::File, Self>(file) {
            Ok(app) => app,
            Err(err) => return Err(yaml_error(err)),
        };
        Ok(app)
    }

    pub fn print(&self) {
        println!("{}", self.generate())
    }

    fn generate_lits<T, G: Generate<T>>(items: &Vec<T>, indent: usize) -> String {
        let mut lits: Vec<String> = vec![];
        for item in items.as_slice() {
            match G::do_generate(item, indent) {
                Some(lit) => lits.push(lit),
                None => continue,
            }
        }
        lits.join("\n")
    }

    fn generate_objects(pipe: &Pipe, indent: usize) -> Option<String> {
        let objects = match pipe.objects {
            Some(ref objects) => objects,
            None => return None,
        };
        let objects_lit = Self::generate_lits::<Object, ObjectGenerator>(objects, indent + 1);
        let name = pipe.get_name();
        let indent_lit = indent_literal(indent);
        Some(format!(
            "{}mod {} {{\n{}\n{}}}",
            indent_lit, name, objects_lit, indent_lit
        ))
    }

    fn generate_all_objects(&self, indent: usize) -> String {
        let mut all_objects_lits: Vec<String> = vec![];
        for pipe in self.pipes.as_slice() {
            match Self::generate_objects(pipe, indent) {
                Some(object_lit) => all_objects_lits.push(object_lit),
                None => continue,
            }
        }
        all_objects_lits.join("\n\n")
    }

    pub fn generate(&self) -> String {
        let pipe_metas = Self::generate_lits::<Pipe, PipeGenerator>(&(self.pipes), 1);
        let all_objects = self.generate_all_objects(1);
        format!(
            "mod {} {{\n{}\n\n{}\n}}",
            self.name, all_objects, pipe_metas
        )
    }

    pub fn validate(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_app() {
        // let manifest_path = "resources/manifest/print_timer_tick.yml";
        // let manifest_path = "resources/manifest/simple_app.yml";
        let manifest_path = "resources/manifest/project_record.yml";
        let app = App::parse(manifest_path).unwrap();
        app.print()
    }
}
