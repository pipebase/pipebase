mod app;
mod constants;
mod context;
mod data;
mod function;
mod meta;
mod package;
mod pipe;
mod utils;

pub use app::*;
pub(crate) use constants::*;
pub(crate) use context::*;
pub(crate) use data::*;
pub(crate) use function::*;
pub use package::*;
pub(crate) use pipe::*;

pub trait Entity {
    fn get_id(&self) -> String;
    fn list_dependency(&self) -> Vec<String> {
        vec![]
    }
    fn to_literal(&self, indent: usize) -> String;
}

pub trait EntityAccept<V: VisitEntity<Self>>: Sized + Entity + Clone {
    fn accept(&self, v: &mut V) {
        v.visit(self)
    }
}

pub trait VisitEntity<E: EntityAccept<Self> + Entity>: Sized {
    fn visit(&mut self, entity: &E);
}
