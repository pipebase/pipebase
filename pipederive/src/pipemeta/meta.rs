use super::Expr;
use super::VisitPipeMeta;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use syn::Attribute;

use crate::constants::PIPE_UPSTREAM_NAME_SEP;
use crate::constants::{
    PIPE_CONFIG_EMPTY_PATH, PIPE_CONFIG_PATH, PIPE_CONFIG_TYPE, PIPE_KIND, PIPE_NAME, PIPE_OUTPUT,
    PIPE_UPSTREAM,
};
use crate::utils::get_meta_string_value_by_meta_path;

/// Pipe configuration type name and path
#[derive(Clone)]
pub struct PipeConfigMeta {
    pub ty: String,
    pub path: Option<String>,
}

impl PipeConfigMeta {
    pub fn get_ty(&self) -> String {
        self.ty.to_owned()
    }

    pub fn get_path(&self) -> String {
        match self.path.to_owned() {
            Some(path) => path,
            None => PIPE_CONFIG_EMPTY_PATH.to_owned(),
        }
    }
}

/// Pipe metadata
pub struct PipeMeta {
    pub name: String,
    pub kind: String,
    pub config_meta: PipeConfigMeta,
    pub output_meta: Option<String>,
    pub upstream_names: Vec<String>,
    pub upstream_output_meta: Option<String>,
    pub downstream_names: Vec<String>,
}

impl PipeMeta {
    pub fn accept<V: VisitPipeMeta>(&self, visitor: &mut V) {
        visitor.visit(self);
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_kind(&self) -> String {
        self.kind.to_owned()
    }

    pub fn get_config_meta(&self) -> PipeConfigMeta {
        self.config_meta.to_owned()
    }

    pub fn get_output_meta(&self) -> Option<String> {
        self.output_meta.to_owned()
    }

    pub fn get_upstream_output_meta(&self) -> Option<String> {
        self.upstream_output_meta.to_owned()
    }

    pub fn get_upstream_names(&self) -> Vec<String> {
        self.upstream_names.to_owned()
    }

    pub fn set_upstream_output_meta(&mut self, upstream_output_meta: Option<String>) {
        let upstream_output_meta = match upstream_output_meta {
            Some(upstream_output_meta) => upstream_output_meta,
            None => panic!("output meta not found in upstream pipe"),
        };
        // upstream pipes should have identical output meta
        match self.upstream_output_meta {
            Some(ref local_upstream_output_meta) => {
                assert!(local_upstream_output_meta.eq(&upstream_output_meta))
            }
            None => self.upstream_output_meta = Some(upstream_output_meta),
        }
    }

    pub fn add_downstream_name(&mut self, downstream_name: &str) {
        self.downstream_names.push(downstream_name.to_owned())
    }

    pub fn get_downstream_names(&self) -> Vec<String> {
        self.downstream_names.to_owned()
    }

    pub fn parse(attribute: &Attribute) -> Self {
        PipeMeta {
            name: Self::parse_name(attribute),
            kind: Self::parse_kind(attribute),
            config_meta: Self::parse_config_meta(attribute),
            output_meta: Self::parse_output_meta(attribute),
            upstream_names: Self::parse_upstream_names(attribute),
            upstream_output_meta: None,
            downstream_names: vec![],
        }
    }

    fn parse_name(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(PIPE_NAME, attribute, true).unwrap()
    }

    fn parse_kind(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(PIPE_KIND, attribute, true).unwrap()
    }

    fn parse_upstream_names(attribute: &Attribute) -> Vec<String> {
        match get_meta_string_value_by_meta_path(PIPE_UPSTREAM, attribute, false) {
            Some(upstream_names) => upstream_names
                .split(PIPE_UPSTREAM_NAME_SEP)
                .map(|n| n.to_owned())
                .collect(),
            None => vec![],
        }
    }

    fn parse_config_meta(attribute: &Attribute) -> PipeConfigMeta {
        let ty = get_meta_string_value_by_meta_path(PIPE_CONFIG_TYPE, attribute, true).unwrap();
        let path = get_meta_string_value_by_meta_path(PIPE_CONFIG_PATH, attribute, false);
        PipeConfigMeta { ty: ty, path: path }
    }

    fn parse_output_meta(attribute: &Attribute) -> Option<String> {
        match get_meta_string_value_by_meta_path(PIPE_OUTPUT, attribute, false) {
            Some(ty) => Some(ty),
            None => None,
        }
    }
}

#[derive(Default)]
pub struct PipeMetas {
    pub pipe_metas: HashMap<String, Rc<RefCell<PipeMeta>>>,
}

impl PipeMetas {
    pub fn parse(attributes: &Vec<Attribute>) -> Self {
        let mut pipe_metas: HashMap<String, Rc<RefCell<PipeMeta>>> = HashMap::new();
        let mut pipe_names = vec![];
        for attribute in attributes {
            let pipe_meta = PipeMeta::parse(&attribute);
            let pipe_name = pipe_meta.get_name();
            pipe_names.push(pipe_name.to_owned());
            pipe_metas.insert(pipe_name.to_owned(), Rc::new(RefCell::new(pipe_meta)));
        }
        for pipe_name in pipe_names.as_slice() {
            let pipe_meta = pipe_metas.get(pipe_name).unwrap().to_owned();
            let upstream_names = pipe_meta.to_owned().deref().borrow().get_upstream_names();
            for upstream_name in upstream_names.as_slice() {
                // upstream pipe register downstream pipe name
                let upstream_pipe_meta = pipe_metas.get(upstream_name).unwrap().to_owned();
                upstream_pipe_meta
                    .deref()
                    .borrow_mut()
                    .add_downstream_name(pipe_name);
                // downstream pipe collect upstream pipe output type and check identical
                let upstream_output_meta = upstream_pipe_meta.deref().borrow().get_output_meta();
                pipe_meta
                    .deref()
                    .borrow_mut()
                    .set_upstream_output_meta(upstream_output_meta)
            }
        }
        PipeMetas {
            pipe_metas: pipe_metas,
        }
    }

    pub fn list_pipe_name(&self) -> Vec<String> {
        let mut pipe_names: Vec<String> = vec![];
        for name in self.pipe_metas.keys() {
            pipe_names.push(name.to_owned())
        }
        pipe_names
    }

    fn visit_pipe_meta<T: VisitPipeMeta>(&self) -> Vec<T> {
        let mut exprs: Vec<T> = vec![];
        for pipe_meta in self.pipe_metas.values() {
            let mut expr = T::default();
            pipe_meta.deref().borrow().accept(&mut expr);
            exprs.push(expr)
        }
        exprs
    }

    // generate expr per pipe meta
    pub fn generate_pipe_meta_exprs<T: VisitPipeMeta + Expr>(&self) -> Vec<String> {
        self.visit_pipe_meta::<T>()
            .iter()
            .map(|e| e.get_expr())
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect()
    }

    fn visit_pipe_metas<T: VisitPipeMeta>(&self) -> T {
        let mut expr = T::default();
        for pipe_meta in self.pipe_metas.values() {
            pipe_meta.deref().borrow().accept(&mut expr)
        }
        expr
    }

    // generate expr based on all pipe metas
    pub fn generate_pipe_metas_expr<T: VisitPipeMeta + Expr>(&self) -> Vec<String> {
        match self.visit_pipe_metas::<T>().get_expr() {
            Some(expr) => vec![expr],
            None => vec![],
        }
    }
}
