use crate::api::utils::indent_literal;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum MetaValue {
    Str(String),
    Int(i32),
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum Meta {
    // Base meta enum
    Path { name: String },
    Value { name: String, meta: MetaValue },
    List { name: String, metas: Vec<Meta> },
    // Extend meta enum
    Derive(Vec<String>),
    Project(HashMap<String, String>),
    Filter(HashMap<String, String>),
    HashKey,
    OrderKey,
    FieldVisit,
}

fn expand_path_list(name: &str, paths: &Vec<String>) -> Meta {
    let mut meta_paths: Vec<Meta> = Vec::new();
    for path in paths {
        meta_paths.push(Meta::Path {
            name: path.to_owned(),
        })
    }
    Meta::List {
        name: name.to_owned(),
        metas: meta_paths,
    }
}

fn meta_str_value(name: &str, value: &str) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Str(value.to_owned()),
    }
}

fn meta_int_value(name: &str, value: &i32) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Int(value.to_owned()),
    }
}

fn expand_str_value_list(name: &str, values: &HashMap<String, String>) -> Meta {
    let mut meta_values: Vec<Meta> = Vec::new();
    for (name, value) in values {
        meta_values.push(meta_str_value(name, value));
    }
    Meta::List {
        name: name.to_owned(),
        metas: meta_values,
    }
}

fn expand_meta(meta: &Meta) -> Meta {
    match meta {
        Meta::Derive(derives) => expand_path_list("derive", derives),
        Meta::Project(projects) => expand_str_value_list("project", projects),
        Meta::Filter(filters) => expand_str_value_list("filter", filters),
        Meta::HashKey => Meta::Path {
            name: "hkey".to_owned(),
        },
        Meta::OrderKey => Meta::Path {
            name: "okey".to_owned(),
        },
        Meta::FieldVisit => Meta::Path {
            name: "visit".to_owned(),
        },
        _ => meta.to_owned(),
    }
}

fn expand_meta_lit(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let expanded_meta = expand_meta(meta);
    let (name, metas) = match expanded_meta {
        Meta::Path { name } => return format!("{}{}", indent_lit, name),
        Meta::Value { name, meta } => match meta {
            MetaValue::Str(value) => return format!(r#"{}{} = "{}""#, indent_lit, name, value),
            MetaValue::Int(value) => return format!("{}{} = {}", indent_lit, name, value),
        },
        Meta::List { name, metas } => (name, metas),
        _ => unreachable!(),
    };
    let mut nested_metas_lits: Vec<String> = vec![];
    for nested_meta in &metas {
        nested_metas_lits.push(expand_meta_lit(nested_meta, indent + 1));
    }
    let nested_metas_lit = nested_metas_lits.join(",\n");
    format!(
        "{}{}(\n{}\n{})",
        indent_lit, name, nested_metas_lit, indent_lit
    )
}

fn meta_to_literal(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let meta_lit = expand_meta_lit(meta, indent + 1);
    format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
}

pub fn metas_to_literal(metas: &Vec<Meta>, indent: usize) -> String {
    let mut metas_literal: Vec<String> = vec![];
    for meta in metas {
        metas_literal.push(meta_to_literal(meta, indent))
    }
    metas_literal.join("\n")
}
