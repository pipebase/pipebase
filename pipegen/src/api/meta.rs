use crate::api::utils::indent_literal;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    Str(String),
    Int(i32),
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum Meta {
    // Base Meta
    Value { name: String, meta: MetaValue },
    List { name: String, metas: Vec<Meta> },
    Path { name: String },
    // Extended Meta
    Derive { derives: Vec<String> },
    Project { project: HashMap<String, String> },
    Filter { filter: HashMap<String, String> },
    AggregateAs { agg: String },
    Tag { tag: String },
}

fn meta_value_str(name: &str, value: &str) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Str(value.to_owned()),
    }
}

fn new_path(name: String) -> Meta {
    Meta::Path { name: name }
}

fn expand_paths(name: &str, paths: &Vec<String>) -> Meta {
    let mut path_metas: Vec<Meta> = Vec::new();
    for path in paths {
        path_metas.push(new_path(path.to_owned()));
    }
    Meta::List {
        name: name.to_owned(),
        metas: path_metas,
    }
}

fn expand_str_values(name: &str, values: &HashMap<String, String>) -> Meta {
    let mut value_metas: Vec<Meta> = Vec::new();
    for (name, value) in values {
        value_metas.push(meta_value_str(name, value));
    }
    Meta::List {
        name: name.to_owned(),
        metas: value_metas,
    }
}

fn expand_meta_lit(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let (name, metas) = match meta {
        Meta::Path { name } => return format!("{}{}", indent_lit, name),
        Meta::Value { name, meta } => match meta {
            MetaValue::Str(value) => return format!(r#"{}{} = "{}""#, indent_lit, name, value),
            MetaValue::Int(value) => return format!("{}{} = {}", indent_lit, name, value),
        },
        Meta::Derive { derives } => {
            let meta = expand_paths("derive", derives);
            return expand_meta_lit(&meta, indent);
        }
        Meta::Project { project } => {
            let meta = expand_str_values("project", project);
            return expand_meta_lit(&meta, indent);
        }
        Meta::Filter { filter } => {
            let meta = expand_str_values("filter", filter);
            return expand_meta_lit(&meta, indent);
        }
        Meta::AggregateAs { agg } => {
            let meta = expand_paths("agg", &vec![agg.to_owned()]);
            return expand_meta_lit(&meta, indent);
        }
        Meta::Tag { tag } => {
            let meta = new_path(tag.to_owned());
            return expand_meta_lit(&meta, indent);
        }
        Meta::List { name, metas } => (name, metas),
    };
    let mut nested_metas_lits: Vec<String> = vec![];
    for nested_meta in metas {
        nested_metas_lits.push(expand_meta_lit(nested_meta, indent + 1));
    }
    let nested_metas_lit = nested_metas_lits.join(",\n");
    format!(
        "{}{}(\n{}\n{})",
        indent_lit, name, nested_metas_lit, indent_lit
    )
}

pub(crate) fn meta_to_literal(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let meta_lit = expand_meta_lit(meta, indent + 1);
    format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
}

pub(crate) fn metas_to_literal(metas: &Vec<Meta>, indent: usize) -> String {
    let mut metas_literal: Vec<String> = vec![];
    for meta in metas {
        metas_literal.push(meta_to_literal(meta, indent))
    }
    metas_literal.join("\n")
}
