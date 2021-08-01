use super::{Expr, VisitContextStoreMeta, VisitErrorHandlerMeta, VisitPipeMeta};

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use syn::Attribute;

use crate::constants::BOOTSTRAP_PIPE_UPSTREAM_NAME_SEP;
use crate::constants::{
    BOOTSTRAP_PIPE_CONFIG_EMPTY_PATH, BOOTSTRAP_PIPE_CONFIG_PATH, BOOTSTRAP_PIPE_CONFIG_TYPE,
    BOOTSTRAP_PIPE_IDENT_SUFFIX, BOOTSTRAP_PIPE_NAME, BOOTSTRAP_PIPE_OUTPUT, BOOTSTRAP_PIPE_TYPE,
    BOOTSTRAP_PIPE_UPSTREAM, CONTEXT_STORE_CONFIG_EMPTY_PATH, CONTEXT_STORE_CONFIG_PATH,
    CONTEXT_STORE_CONFIG_TYPE, CONTEXT_STORE_IDENT_SUFFIX, CONTEXT_STORE_NAME,
    ERROR_HANDLER_CONFIG_PATH, ERROR_HANDLER_CONFIG_TYPE,
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
    pub ident: String,
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

    pub fn get_ident(&self) -> &String {
        &self.ident
    }

    pub fn get_ty(&self) -> &String {
        &self.ty
    }

    pub fn get_config_meta(&self) -> &PipeConfigMeta {
        &self.config_meta
    }

    pub fn get_output_type_name(&self) -> Option<&String> {
        self.output_type_name.as_ref()
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
                assert!(
                    local_upstream_output_type_name.eq(&upstream_output_type_name),
                    "upstream output conflict, found {} != {}",
                    local_upstream_output_type_name,
                    upstream_output_type_name
                );
            }
            None => self.upstream_output_type_name = Some(upstream_output_type_name),
        }
    }

    pub fn add_downstream_names(&mut self, downstream_names: Vec<String>) {
        self.downstream_names.extend(downstream_names)
    }

    pub fn get_downstream_names(&self) -> &Vec<String> {
        &self.downstream_names
    }

    pub fn parse(attribute: &Attribute, ident_location: &str) -> Self {
        let name = Self::parse_name(attribute, ident_location);
        let ident = format!("{}{}", name, BOOTSTRAP_PIPE_IDENT_SUFFIX);
        PipeMeta {
            name,
            ident,
            ty: Self::parse_ty(attribute, ident_location),
            config_meta: Self::parse_config_meta(attribute, ident_location),
            output_type_name: Self::parse_output_meta(attribute),
            upstream_names: Self::parse_upstream_names(attribute),
            upstream_output_type_name: None,
            downstream_names: vec![],
        }
    }

    fn parse_name(attribute: &Attribute, ident_location: &str) -> String {
        get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_NAME,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap()
    }

    fn parse_ty(attribute: &Attribute, ident_location: &str) -> String {
        get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_TYPE,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap()
    }

    fn parse_upstream_names(attribute: &Attribute) -> Vec<String> {
        match get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_UPSTREAM,
            &get_meta(attribute),
            false,
            "",
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

    fn parse_config_meta(attribute: &Attribute, ident_location: &str) -> PipeConfigMeta {
        let ty = get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_CONFIG_TYPE,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap();
        let path = get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_CONFIG_PATH,
            &get_meta(attribute),
            false,
            ident_location,
        );
        PipeConfigMeta { ty: ty, path: path }
    }

    fn parse_output_meta(attribute: &Attribute) -> Option<String> {
        match get_meta_string_value_by_meta_path(
            BOOTSTRAP_PIPE_OUTPUT,
            &get_meta(attribute),
            false,
            "",
        ) {
            Some(ty) => Some(ty),
            None => None,
        }
    }

    pub fn generate_pipe_meta_expr<T: VisitPipeMeta + Expr>(&self) -> Option<String> {
        let mut visitor = T::default();
        self.accept(&mut visitor);
        visitor.get_expr()
    }
}

#[derive(Default)]
pub struct PipeMetas {
    pub pipe_metas: HashMap<String, PipeMeta>,
}

impl PipeMetas {
    pub fn parse(attributes: &Vec<Attribute>, ident_location: &str) -> Self {
        let mut pipe_metas: HashMap<String, PipeMeta> = HashMap::new();
        let mut pipe_names: HashSet<String> = HashSet::new();
        let mut pipe_output_type_names: HashMap<String, Option<String>> = HashMap::new();
        let mut downstream_pipe_names: HashMap<String, Vec<String>> = HashMap::new();
        let mut upstream_pipe_names: HashMap<String, HashSet<String>> = HashMap::new();
        for attribute in attributes {
            let ref pipe_meta = PipeMeta::parse(&attribute, ident_location);
            let pipe_name = pipe_meta.get_name();
            assert!(
                pipe_names.insert(pipe_name.to_owned()),
                "duplicated pipe name {}",
                pipe_name
            );
            pipe_metas.insert(pipe_name.to_owned(), pipe_meta.to_owned());
            // collect output type per pipe - channel ty
            pipe_output_type_names.insert(
                pipe_name.to_owned(),
                pipe_meta.get_output_type_name().cloned(),
            );
            // collect upstream pipe for input lookup - channel rx
            upstream_pipe_names.insert(
                pipe_name.to_owned(),
                HashSet::from_iter(pipe_meta.get_upstream_names().to_owned()),
            );
            // collect downstream pipe - channel tx
            for upstream_pipe_name in pipe_meta.get_upstream_names() {
                let ds = downstream_pipe_names
                    .entry(upstream_pipe_name.to_owned())
                    .or_insert(vec![]);
                ds.push(pipe_name.to_owned());
            }
        }
        for pipe_name in &pipe_names {
            let pipe_meta = pipe_metas.get_mut(pipe_name).expect("pipe meta");
            // connect downstream pipe
            pipe_meta.add_downstream_names(
                downstream_pipe_names
                    .get(pipe_name)
                    .cloned()
                    .unwrap_or(vec![]),
            );
            // setup upstream output as input type for channel
            for upstream_pipe_name in upstream_pipe_names.get(pipe_name).expect("upstreams") {
                let upstream_output_type_name = pipe_output_type_names
                    .get(upstream_pipe_name)
                    .expect(&format!(
                        "upstream pipe {} does not exists",
                        upstream_pipe_name
                    ))
                    .to_owned()
                    .expect(&format!(
                        "output type not found in upstream pipe {}",
                        upstream_pipe_name
                    ));
                pipe_meta.set_upstream_output_type_name(upstream_output_type_name);
            }
        }
        PipeMetas {
            pipe_metas: pipe_metas,
        }
    }

    pub fn list_pipe_ident(&self) -> Vec<String> {
        self.pipe_metas
            .values()
            .into_iter()
            .map(|k| k.get_ident().to_owned())
            .collect()
    }

    // generate expr per pipe meta
    pub fn generate_pipe_meta_exprs<T: VisitPipeMeta + Expr>(&self) -> Vec<String> {
        self.pipe_metas
            .values()
            .into_iter()
            .filter_map(|meta| meta.generate_pipe_meta_expr::<T>())
            .collect()
    }

    pub fn accept<T: VisitPipeMeta>(&self, visitor: &mut T) {
        for pipe_meta in self.pipe_metas.values() {
            pipe_meta.accept(visitor)
        }
    }
}

pub struct ContextStoreConfigMeta {
    ty: String,
    path: Option<String>,
}

impl ContextStoreConfigMeta {
    pub fn get_ty(&self) -> String {
        self.ty.to_owned()
    }

    pub fn get_path(&self) -> String {
        match self.path.to_owned() {
            Some(path) => path,
            None => CONTEXT_STORE_CONFIG_EMPTY_PATH.to_owned(),
        }
    }
}

pub struct ContextStoreMeta {
    name: String,
    ident: String,
    config_meta: ContextStoreConfigMeta,
    pipe_idents: Vec<String>,
}

impl ContextStoreMeta {
    pub fn accept<V: VisitContextStoreMeta>(&self, visitor: &mut V) {
        visitor.visit(self)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_ident(&self) -> &String {
        &self.ident
    }

    pub fn set_pipes(&mut self, pipe_idents: Vec<String>) {
        self.pipe_idents = pipe_idents
    }

    pub fn get_pipes(&self) -> Vec<String> {
        self.pipe_idents.to_owned()
    }

    pub fn get_config_meta(&self) -> &ContextStoreConfigMeta {
        &self.config_meta
    }

    pub fn parse(attribute: &Attribute, ident_location: &str) -> Self {
        let name = Self::parse_name(attribute, ident_location);
        let ident = format!("{}{}", name, CONTEXT_STORE_IDENT_SUFFIX);
        ContextStoreMeta {
            name,
            ident,
            config_meta: Self::parse_config_meta(attribute, ident_location),
            pipe_idents: Vec::new(),
        }
    }

    fn parse_name(attribute: &Attribute, ident_location: &str) -> String {
        get_meta_string_value_by_meta_path(
            CONTEXT_STORE_NAME,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap()
    }

    fn parse_config_meta(attribute: &Attribute, ident_location: &str) -> ContextStoreConfigMeta {
        let ty = get_meta_string_value_by_meta_path(
            CONTEXT_STORE_CONFIG_TYPE,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap();
        let path = get_meta_string_value_by_meta_path(
            CONTEXT_STORE_CONFIG_PATH,
            &get_meta(attribute),
            false,
            "",
        );
        ContextStoreConfigMeta { ty: ty, path: path }
    }

    pub fn generate_cstore_meta_expr<V: VisitContextStoreMeta + Expr>(&self) -> Option<String> {
        let mut visitor = V::default();
        self.accept(&mut visitor);
        visitor.get_expr()
    }
}

pub struct ContextStoreMetas {
    metas: Vec<ContextStoreMeta>,
}

impl ContextStoreMetas {
    pub fn parse(attributes: &Vec<Attribute>, ident_location: &str) -> Self {
        let metas: Vec<ContextStoreMeta> = attributes
            .into_iter()
            .map(|attribute| ContextStoreMeta::parse(attribute, ident_location))
            .collect();
        ContextStoreMetas { metas }
    }

    pub fn add_pipes(&mut self, pipe_idents: Vec<String>) {
        for meta in &mut self.metas {
            meta.set_pipes(pipe_idents.to_owned())
        }
    }

    pub fn generate_cstore_meta_exprs<V: VisitContextStoreMeta + Expr>(&self) -> Vec<String> {
        self.metas
            .iter()
            .filter_map(|meta| meta.generate_cstore_meta_expr::<V>())
            .collect()
    }

    pub fn accept<V: VisitContextStoreMeta>(&self, visitor: &mut V) {
        for meta in &self.metas {
            visitor.visit(meta)
        }
    }
}

pub struct ErrorHandlerConfigMeta {
    ty: String,
    path: Option<String>,
}

impl ErrorHandlerConfigMeta {
    pub fn get_ty(&self) -> String {
        self.ty.to_owned()
    }

    pub fn get_path(&self) -> String {
        match self.path.to_owned() {
            Some(path) => path,
            None => CONTEXT_STORE_CONFIG_EMPTY_PATH.to_owned(),
        }
    }
}

pub struct ErrorHandlerMeta {
    config_meta: ErrorHandlerConfigMeta,
    pipe_idents: Vec<String>,
}

impl ErrorHandlerMeta {
    pub fn accept<V: VisitErrorHandlerMeta>(&self, visitor: &mut V) {
        visitor.visit(self)
    }

    pub fn set_pipes(&mut self, pipe_idents: Vec<String>) {
        self.pipe_idents = pipe_idents;
    }

    pub fn get_pipes(&self) -> Vec<String> {
        self.pipe_idents.to_owned()
    }

    pub fn get_config_meta(&self) -> &ErrorHandlerConfigMeta {
        &self.config_meta
    }

    pub fn parse(attribute: Option<&Attribute>, ident_location: &str) -> Option<Self> {
        let attribute = match attribute {
            Some(attribute) => attribute,
            None => return None,
        };
        let config_meta = Self::parse_config_meta(attribute, ident_location);
        Some(ErrorHandlerMeta {
            config_meta,
            pipe_idents: Vec::new(),
        })
    }

    fn parse_config_meta(attribute: &Attribute, ident_location: &str) -> ErrorHandlerConfigMeta {
        let ty = get_meta_string_value_by_meta_path(
            ERROR_HANDLER_CONFIG_TYPE,
            &get_meta(attribute),
            true,
            ident_location,
        )
        .unwrap();
        let path = get_meta_string_value_by_meta_path(
            ERROR_HANDLER_CONFIG_PATH,
            &get_meta(attribute),
            false,
            "",
        );
        ErrorHandlerConfigMeta { ty: ty, path: path }
    }

    pub fn generate_error_handler_meta_expr<V: VisitErrorHandlerMeta + Expr>(
        &self,
    ) -> Option<String> {
        let mut visitor = V::default();
        self.accept(&mut visitor);
        visitor.get_expr()
    }
}
