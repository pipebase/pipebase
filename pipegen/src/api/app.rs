use crate::api::pipe::Pipe;
use crate::api::schema::Structure;
use crate::error::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct App {
    pub name: String,
    pub pipes: Vec<Pipe>,
    pub structures: Vec<Structure>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_app() {
        let app = App::parse("resources/manifest/simple_app.yml").unwrap();
        println!("{:#?}", app)
    }
}
