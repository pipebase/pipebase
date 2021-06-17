use super::Object;
use crate::api::pipe::Pipe;
use crate::error::*;
use crate::operation::{Generate, ObjectGenerator, PipeGenerator, PipeIdValidator, Validate};
use serde::Deserialize;

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
        let objects_lit = Self::generate_lits::<Object, ObjectGenerator>(objects, indent);
        Some(objects_lit)
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

    fn validate_entity<T, V: Validate<T>>(items: &Vec<T>, location: &str) -> Result<()> {
        V::do_validate(items, location)
    }

    pub fn validate(&self) -> Result<()> {
        Self::validate_entity::<Pipe, PipeIdValidator>(&self.pipes, "pipes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_object() {
        let manifest_path = "resources/manifest/complex_object.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_print_timer_tick() {
        let manifest_path = "resources/manifest/print_timer_tick.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn test_project() {
        let manifest_path = "resources/manifest/project_record.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.print()
    }

    #[test]
    fn pipe_name_bad_case() {
        let manifest_path = "resources/manifest/pipe_name_bad_case.yml";
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }
}
