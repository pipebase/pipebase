use crate::api::pipe::Pipe;
use crate::api::EntityAccept;
use crate::error::*;
use crate::operation::{Generate, PipeGenerator};
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

    pub fn generate(&self) -> String {
        let mut pipe_metas_lits: Vec<String> = vec![];
        // generate pipe metas
        for pipe in self.pipes.as_slice() {
            let mut pipe_metas_generator = PipeGenerator {
                indent: 2,
                pipe: None,
            };
            pipe.accept(&mut pipe_metas_generator);
            pipe_metas_lits.push(pipe_metas_generator.generate().unwrap())
        }
        pipe_metas_lits.join("\n")
    }

    pub fn validate(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_app() {
        let app = App::parse("resources/manifest/simple_app.yml").unwrap();
        // println!("{:#?}", app)
        app.print()
    }
}
