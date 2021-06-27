use super::Expr;
use super::VisitPipeMeta;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use syn::Attribute;

use crate::constants::BOOTSTRAP_PIPE_UPSTREAM_NAME_SEP;
use crate::constants::{
    BOOTSTRAP_PIPE_CONFIG_EMPTY_PATH, BOOTSTRAP_PIPE_CONFIG_PATH, BOOTSTRAP_PIPE_CONFIG_TYPE,
    BOOTSTRAP_PIPE_NAME, BOOTSTRAP_PIPE_OUTPUT, BOOTSTRAP_PIPE_TYPE, BOOTSTRAP_PIPE_UPSTREAM,
};
use crate::utils::{get_meta, get_meta_string_value_by_meta_path};

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
            None => BOOTSTRAP_PIPE_CONFIG_EMPTY_PATH.to_owned(),
        }
    }
}

/// Pipe metadata
#[derive(Clone)]
pub struct PipeMeta {
    pub name: String,
    pub ty: String,
    pub config_meta: PipeConfigMeta,
    pub output_type_name: Option<String>,
    pub upstream_names: Vec<String>,
    pub upstream_output_type_name: Option<String>,
    pub downstream_names: Vec<String>,
}

impl PipeMeta {
    pub fn accept<V: VisitPipeMeta>(&self, visitor: &mut V) {
        visitor.visit(self);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_ty(&self) -> &String {
        &self.ty
    }

    pub fn get_config_meta(&self) -> &PipeConfigMeta {
        &self.config_meta
    }

    pub fn get_output_type_name(&self) -> &Option<String> {
        &self.output_type_name
    }

    pub fn get_upstream_output_type_name(&self) -> Option<String> {
        self.upstream_output_type_name.to_owned()
    }

    pub fn get_upstream_names(&self) -> &Vec<String> {
        &self.upstream_names
    }

    pub fn set_upstream_output_type_name(&mut self, upstream_output_type_name: String) {
        // upstream pipes should have identical output meta
        match self.upstream_output_type_name {
            Some(ref local_upstream_output_type_name) => {
                if !local_upstream_output_type_name.eq(&upstream_output_type_name) {
                    panic!(
                        "upstream output conflict, found {} != {}",
                        local_upstream_output_type_name, upstream_output_type_name
                    )
                }
            }
            None => self.upstream_output_type_name = Some(upstream_output_type_name),
        }
    }

    pub fn add_downstream_name(&mut self, downstream_name: String) {
        self.downstream_names.push(downstream_name)
    }

    pub fn get_downstream_names(&self) -> &Vec<String> {
        &self.downstream_names
    }

    pub fn parse(attribute: &Attribute) -> Self {
        PipeMeta {
            name: Self::parse_name(attribute),
            ty: Self::parse_ty(attribute),
            config_meta: Self::parse_config_meta(attribute),
            output_type_name: Self::parse_output_meta(attribute),
            upstream_names: Self::parse_upstream_names(attribute),
            upstream_output_type_name: None,
            downstream_names: vec![],
        }
    }

    fn parse_name(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(BOOTSTRAP_PIPE_NAME, &get_meta(attribute), true).unwrap()
    }

    fn parse_ty(attribute: &Attribute) -> String {
        get_meta_string_value_by_meta_path(BOOTSTRAP_PIPE_TYPE, &get_meta(attribute), true).unwrap()
    }

    fn parse_upstream_names(attribute: &Attribute) -> Vec<String> {
        match get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_UPSTREAM,
            &get_meta(attribute),
            false,
        ) {
            Some(upstream_names) => {
                // split into vector of upstreams
                upstream_names
                    .split(BOOTSTRAP_PIPE_UPSTREAM_NAME_SEP)
                    .map(|n| {
                        let mut n = n.to_owned();
                        // clean whitespace after split
                        n.retain(|c| !c.is_whitespace());
                        n
                    })
                    .collect()
            }
            None => vec![],
        }
    }

    fn parse_config_meta(attribute: &Attribute) -> PipeConfigMeta {
        let ty = get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_CONFIG_TYPE,
            &get_meta(attribute),
            true,
        )
        .unwrap();
        let path = get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_CONFIG_PATH,
            &get_meta(attribute),
            false,
        );
        PipeConfigMeta { ty: ty, path: path }
    }

    fn parse_output_meta(attribute: &Attribute) -> Option<String> {
        match get_meta_string_value_by_meta_path(BOOTSTRAP_PIPE_OUTPUT, &get_meta(attribute), false)
        {
            Some(ty) => Some(ty),
            None => None,
        }
    }
}

#[derive(Default)]
pub struct PipeMetas {
    pub pipe_metas: HashMap<String, PipeMeta>,
}

impl PipeMetas {
    pub fn parse(attributes: &Vec<Attribute>) -> Self {
        let mut pipe_metas: HashMap<String, PipeMeta> = HashMap::new();
        let mut pipe_names: HashSet<String> = HashSet::new();
        let mut pipe_output_type_names: HashMap<String, Option<String>> = HashMap::new();
        let mut downstream_pipe_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut upstream_pipe_names: HashMap<String, HashSet<String>> = HashMap::new();
        for attribute in attributes {
            let ref pipe_meta = PipeMeta::parse(&attribute);
            let pipe_name = pipe_meta.get_name();
            if !pipe_names.insert(pipe_name.to_owned()) {
                panic!("duplicated pipe name {}", pipe_name)
            }
            pipe_metas.insert(pipe_name.to_owned(), pipe_meta.to_owned());
            // collect output type per pipe - channel ty
            pipe_output_type_names.insert(
                pipe_name.to_owned(),
                pipe_meta.get_output_type_name().to_owned(),
            );
            // collect upstream pipe for input loopup - channel rx
            upstream_pipe_names.insert(
                pipe_name.to_owned(),
                HashSet::from_iter(pipe_meta.get_upstream_names().to_owned()),
            );
            // collect downstream pipe - channel tx
            for upstream_pipe_name in pipe_meta.get_upstream_names() {
                if !downstream_pipe_names.contains_key(upstream_pipe_name) {
                    downstream_pipe_names
                        .insert(upstream_pipe_name.to_owned(), vec![pipe_name.to_owned()]);
                } else {
                    downstream_pipe_names
                        .get_mut(upstream_pipe_name)
                        .unwrap()
                        .push(pipe_name.to_owned());
                }
            }
        }
        for pipe_name in &pipe_names {
            let pipe_meta = pipe_metas.get_mut(pipe_name).unwrap();
            // connect downstream pipe
            if downstream_pipe_names.contains_key(pipe_name) {
                for downstream_pipe_name in downstream_pipe_names.get(pipe_name).unwrap() {
                    pipe_meta.add_downstream_name(downstream_pipe_name.to_owned())
                }
            }
            // setup upstream output as input type for channel
            for upstream_pipe_name in upstream_pipe_names.get(pipe_name).unwrap() {
                let upstream_output_type_name = match pipe_output_type_names.get(upstream_pipe_name)
                {
                    Some(upstream_output_type_name) => upstream_output_type_name,
                    None => panic!("upstream pipe {} does not exists", upstream_pipe_name),
                };
                match upstream_output_type_name {
                    Some(upstream_output_type_name) => pipe_meta
                        .set_upstream_output_type_name(upstream_output_type_name.to_owned()),
                    None => panic!(
                        "output type not found in upstream pipe {}",
                        upstream_pipe_name
                    ),
                }
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
            pipe_meta.accept(&mut expr);
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
            pipe_meta.accept(&mut expr)
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
