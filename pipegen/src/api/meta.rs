use crate::api::utils::indent_literal;
use serde::Deserialize;

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum MetaValue {
    Str { value: String },
    Int { value: i32 },
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum Meta {
    Path { name: String },
    Value { name: String, meta: MetaValue },
    List { name: String, metas: Vec<Meta> },
}

pub fn expand_meta_lit(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let (name, metas) = match meta.to_owned() {
        Meta::Path { name } => return format!("{}{}", indent_lit, name),
        Meta::Value { name, meta } => match meta {
            MetaValue::Str { value } => return format!(r#"{}{} = "{}""#, indent_lit, name, value),
            MetaValue::Int { value } => return format!("{}{} = {}", indent_lit, name, value),
        },
        Meta::List { name, metas } => (name, metas),
    };
    let mut nested_metas_lits: Vec<String> = vec![];
    for nested_meta in metas.as_slice() {
        nested_metas_lits.push(expand_meta_lit(nested_meta, indent + 1));
    }
    let nested_metas_lit = nested_metas_lits.join(",\n");
    format!(
        "{}{}(\n{}\n{})",
        indent_lit, name, nested_metas_lit, indent_lit
    )
}

fn to_literal(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let meta_lit = expand_meta_lit(meta, indent + 1);
    format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
}

pub fn metas_to_literal(metas: &Vec<Meta>, indent: usize) -> String {
    let mut metas_literal: Vec<String> = vec![];
    for meta in metas {
        metas_literal.push(to_literal(meta, indent))
    }
    metas_literal.join("\n")
}
