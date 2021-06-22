use std::fmt::Display;

use crate::api::utils::indent_literal;
use crate::api::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::data::DataField;

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum PipeType {
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

#[derive(Deserialize, Debug, Clone)]
pub struct PipeConfig {
    pub ty: String,
    pub path: Option<String>,
}

impl PipeConfig {
    pub fn get_path(&self) -> Option<String> {
        self.path.to_owned()
    }
    pub fn get_config_type(&self) -> String {
        self.ty.to_owned()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pipe {
    pub name: String,
    pub ty: PipeType,
    pub config: PipeConfig,
    // upstream pipe names
    pub upstreams: Option<Vec<String>>,
    // output data type
    pub output: Option<DataField>,
}

impl Pipe {
    pub fn is_source(&self) -> bool {
        match &self.ty {
            PipeType::Listener | PipeType::Poller => true,
            _ => false,
        }
    }

    pub fn get_name_meta_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        format!(r#"{}name = "{}""#, indent_lit, self.name)
    }

    pub fn get_type_meta_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        format!(r#"{}ty = "{}""#, indent_lit, self.ty)
    }

    pub fn get_config_meta_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let config_ty_meta_lit = format!(r#"ty = "{}""#, self.config.get_config_type());
        let config_meta_lit = match self.config.get_path() {
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

    pub fn get_upstream_meta_literal(&self, indent: usize) -> Option<String> {
        let upstreams = match self.upstreams {
            Some(ref upstreams) => upstreams,
            None => return None,
        };
        match upstreams.is_empty() {
            false => {
                let indent_lit = indent_literal(indent);
                Some(format!(
                    r#"{}upstream = "{}""#,
                    indent_lit,
                    upstreams.join(", ")
                ))
            }
            true => None,
        }
    }

    pub fn get_output_data_type_meta_literal(&self, indent: usize) -> Option<String> {
        let output_data_type = match self.output {
            Some(ref output_data_type) => output_data_type,
            None => return None,
        };
        Some(format!(
            r#"{}output = "{}""#,
            indent_literal(indent),
            output_data_type.to_literal(0)
        ))
    }
}

impl Entity for Pipe {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.upstreams {
            Some(ref upstreams) => upstreams.to_owned(),
            None => vec![],
        }
    }

    // to pipe meta
    fn to_literal(&self, indent: usize) -> String {
        let mut meta_lits = vec![];
        meta_lits.push(self.get_name_meta_literal(indent + 1));
        meta_lits.push(self.get_type_meta_literal(indent + 1));
        meta_lits.push(self.get_config_meta_literal(indent + 1));
        match self.get_upstream_meta_literal(indent + 1) {
            Some(upstream_literal) => meta_lits.push(upstream_literal),
            None => (),
        };
        match self.get_output_data_type_meta_literal(indent + 1) {
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

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, ty: {}", self.name, self.ty)
    }
}

impl<V: VisitEntity<Pipe>> EntityAccept<V> for Pipe {}
