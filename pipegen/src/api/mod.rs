mod app;
mod constants;
mod pipe;
mod schema;
mod utils;

pub trait Entity {
    fn get_name(&self) -> String;
    fn list_dependency(&self) -> Vec<String>;
    fn to_literal(&self, indent: usize) -> String;
}

pub trait EntityAccept<V: VisitEntity<Self>>: Sized {
    fn accept(&self, v: &mut V) {
        v.visit(self)
    }
}

pub trait VisitEntity<E: EntityAccept<Self>>: Sized {
    fn visit(&mut self, entity: &E);
}
