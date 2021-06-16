use crate::api::utils::indent_literal;
use crate::api::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::data::{DataType, Object};

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum PipeKind {
    #[strum(to_string = "listener")]
    Listener,
    #[strum(to_string = "poller")]
    Poller,
    #[strum(to_string = "mapper")]
    Mapper,
    #[strum(to_string = "collector")]
    Collector,
    #[strum(to_string = "selector")]
    Selector,
    #[strum(to_string = "exporter")]
    Exporter,
}

#[derive(Deserialize, Debug)]
pub struct PipeConfig {
    pub config_type: String,
    pub path: Option<String>,
}

impl PipeConfig {
    pub fn get_path(&self) -> Option<String> {
        self.path.to_owned()
    }
    pub fn get_config_type(&self) -> String {
        self.config_type.to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct Pipe {
    pub name: String,
    pub kind: PipeKind,
    pub pipe_config: PipeConfig,
    pub upstream_pipe_name: Option<String>,
    pub output_data_type: Option<DataType>,
    pub objects: Option<Vec<Object>>,
}

impl Pipe {
    pub fn get_name_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        format!(r#"{}name = "{}""#, indent_lit, self.name)
    }

    pub fn get_kind_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        format!(r#"{}kind = "{}""#, indent_lit, self.kind)
    }

    pub fn get_config_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let config_ty_meta_lit = format!(r#"ty = "{}""#, self.pipe_config.get_config_type());
        let config_meta_lit = match self.pipe_config.get_path() {
            Some(path) => {
                let config_path_lit = format!(r#"path = "{}""#, path);
                format!(
                    "{}config({}, {})",
                    indent_lit, config_ty_meta_lit, config_path_lit
                )
            }
            None => format!("{}config({})", indent_lit, config_ty_meta_lit),
        };
        config_meta_lit
    }

    pub fn get_upstream_literal(&self, indent: usize) -> Option<String> {
        match self.upstream_pipe_name.to_owned() {
            Some(upstream_pipe_name) => {
                let indent_lit = indent_literal(indent);
                Some(format!(
                    r#"{}upstream = "{}""#,
                    indent_lit, upstream_pipe_name
                ))
            }
            None => None,
        }
    }

    pub fn get_output_data_type_literal(&self, indent: usize) -> Option<String> {
        match self.output_data_type.to_owned() {
            Some(output_data) => {
                let indent_lit = indent_literal(indent);
                // let output_mod_lit = format!(r#"module = {}"#, self.name);
                let output_ty_lit = format!(r#"ty = "{}""#, output_data.get_data_type_literal(0));
                Some(format!("{}output({})", indent_lit, output_ty_lit))
            }
            None => None,
        }
    }
}

impl Entity for Pipe {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.upstream_pipe_name.to_owned() {
            Some(pipe_name) => vec![pipe_name],
            None => vec![],
        }
    }

    // to pipe meta
    fn to_literal(&self, indent: usize) -> String {
        let mut meta_lits = vec![];
        meta_lits.push(self.get_name_literal(indent + 1));
        meta_lits.push(self.get_kind_literal(indent + 1));
        meta_lits.push(self.get_config_literal(indent + 1));
        match self.get_upstream_literal(indent + 1) {
            Some(upstream_literal) => meta_lits.push(upstream_literal),
            None => (),
        };
        match self.get_output_data_type_literal(indent + 1) {
            Some(output_ty_literal) => meta_lits.push(output_ty_literal),
            None => (),
        };
        let meta_lits_join = meta_lits.join(",\n");
        let indent_lit = indent_literal(indent);
        format!(
            "{}#[pipe(\n{}\n{})]",
            indent_lit, meta_lits_join, indent_lit
        )
    }
}

impl<V: VisitEntity<Pipe>> EntityAccept<V> for Pipe {}
