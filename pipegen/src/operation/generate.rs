use crate::api::Entity;
use crate::api::Pipe;
use crate::api::VisitEntity;
pub trait Generate {
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

impl Generate for PipeGenerator {
    fn generate(&self) -> Option<String> {
        let pipe = match self.pipe.to_owned() {
            Some(t) => t,
            None => return None,
        };
        Some(pipe.to_literal(self.indent))
    }
}
