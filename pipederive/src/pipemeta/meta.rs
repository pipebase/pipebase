use super::Expr;
use super::{ChannelExpr, PipeExpr, VisitPipeMeta};

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use syn::Attribute;

use crate::constants::{
    PIPE_CONFIG_PATH, PIPE_CONFIG_TYPE, PIPE_KIND, PIPE_NAME, PIPE_OUTPUT_MODULE, PIPE_OUTPUT_TYPE,
    PIPE_UPSTREAM,
};
use crate::utils::get_meta_string_value_by_meta_path;

/// Pipe configuration type name and path
#[derive(Clone)]
pub struct PipeConfigMeta {
    pub ty: String,
    pub path: String,
}

impl PipeConfigMeta {
    pub fn get_ty(&self) -> String {
        self.ty.to_owned()
    }

    pub fn get_path(&self) -> String {
        self.path.to_owned()
    }
}

#[derive(Clone)]
pub struct PipeOutputMeta {
    pub module: Option<String>,
    pub ty: String,
}

impl PipeOutputMeta {
    pub fn get_module(&self) -> Option<String> {
        self.module.to_owned()
    }
    pub fn get_ty(&self) -> String {
        self.ty.to_owned()
    }
    pub fn get_path(&self) -> String {
        let module = match self.get_module() {
            Some(module) => module,
            None => self.get_ty(),
        };
        format!("{}::{}", module, self.get_ty())
    }
}

/// Pipe metadata
pub struct PipeMeta {
    pub name: String,
    pub kind: String,
    pub config_meta: PipeConfigMeta,
    pub output_meta: Option<PipeOutputMeta>,
    pub upstream_name: Option<String>,
    pub upstream_meta: Weak<RefCell<PipeMeta>>,
    pub downstream_metas: Vec<Rc<RefCell<PipeMeta>>>,
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

    pub fn get_output_meta(&self) -> Option<PipeOutputMeta> {
        self.output_meta.to_owned()
    }

    pub fn get_upstream_meta(&self) -> Option<Rc<RefCell<PipeMeta>>> {
        self.upstream_meta.upgrade()
    }

    pub fn get_upstream_name(&self) -> Option<String> {
        self.upstream_name.to_owned()
    }

    pub fn set_upstream_meta(&mut self, upstream_meta: Rc<RefCell<PipeMeta>>) {
        *self.upstream_meta.borrow_mut() = Rc::downgrade(&upstream_meta)
    }

    pub fn add_downstream_meta(&mut self, downstream_meta: Rc<RefCell<PipeMeta>>) {
        self.downstream_metas.push(downstream_meta)
    }

    pub fn get_downstream_metas(&self) -> Vec<Rc<RefCell<PipeMeta>>> {
        self.downstream_metas.to_owned()
    }

    pub fn parse(attribute: &Attribute) -> Self {
        PipeMeta {
            name: Self::parse_name(attribute),
            kind: Self::parse_kind(attribute),
            config_meta: Self::parse_config_meta(attribute),
            output_meta: Self::parse_output_meta(attribute),
            upstream_name: Self::parse_upstream_name(attribute),
            upstream_meta: Weak::new(),
            downstream_metas: vec![],
        }
    }

    fn parse_name(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(PIPE_NAME, attribute, true).unwrap()
    }

    fn parse_kind(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(PIPE_KIND, attribute, true).unwrap()
    }

    fn parse_upstream_name(attribute: &Attribute) -> Option<String> {
        get_meta_string_value_by_meta_path(PIPE_UPSTREAM, attribute, false)
    }

    fn parse_config_meta(attribute: &Attribute) -> PipeConfigMeta {
        let ty = get_meta_string_value_by_meta_path(PIPE_CONFIG_TYPE, attribute, true).unwrap();
        let path = get_meta_string_value_by_meta_path(PIPE_CONFIG_PATH, attribute, true).unwrap();
        PipeConfigMeta { ty: ty, path: path }
    }

    fn parse_output_meta(attribute: &Attribute) -> Option<PipeOutputMeta> {
        let ty = match get_meta_string_value_by_meta_path(PIPE_OUTPUT_TYPE, attribute, false) {
            Some(ty) => ty,
            None => return None,
        };
        let module = get_meta_string_value_by_meta_path(PIPE_OUTPUT_MODULE, attribute, false);
        Some(PipeOutputMeta {
            ty: ty,
            module: module,
        })
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
            let upstream_name = pipe_meta.to_owned().deref().borrow().get_upstream_name();
            let upstream_name = match upstream_name {
                Some(upstream_name) => upstream_name,
                None => continue,
            };
            // connect upstream and downstream pipe
            let upstream_pipe_meta = pipe_metas.get(&upstream_name).unwrap().to_owned();
            upstream_pipe_meta
                .deref()
                .borrow_mut()
                .add_downstream_meta(pipe_meta.to_owned());
            pipe_meta
                .to_owned()
                .deref()
                .borrow_mut()
                .set_upstream_meta(upstream_pipe_meta.to_owned())
        }
        PipeMetas {
            pipe_metas: pipe_metas,
        }
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
