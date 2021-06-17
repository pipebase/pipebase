use crate::api::utils::indent_literal;
use crate::api::Entity;
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum Meta {
    Path { name: String },
    NameStringValue { name: String, value: String },
    NameIntegerValue { name: String, value: i32 },
    NameListMeta { name: String, value: Vec<Meta> },
}

#[derive(Clone, Deserialize, Debug)]
pub struct Attribute {
    pub meta: Meta,
}

impl Attribute {
    pub fn expand_meta_lit(meta: &Meta, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let (name, value) = match meta.to_owned() {
            Meta::Path { name } => return format!("{}{}", indent_lit, name),
            Meta::NameStringValue { name, value } => {
                return format!(r#"{}{} = "{}""#, indent_lit, name, value)
            }
            Meta::NameIntegerValue { name, value } => {
                return format!("{}{} = {}", indent_lit, name, value)
            }
            Meta::NameListMeta { name, value } => (name, value),
        };
        let mut nested_metas_lits: Vec<String> = vec![];
        for nested_meta in value.as_slice() {
            nested_metas_lits.push(Attribute::expand_meta_lit(nested_meta, indent + 1));
        }
        let nested_metas_lit = nested_metas_lits.join(",\n");
        format!(
            "{}{}(\n{}\n{})",
            indent_lit, name, nested_metas_lit, indent_lit
        )
    }
}

impl Entity for Attribute {
    fn get_name(&self) -> String {
        match self.meta.to_owned() {
            Meta::Path { name } => name,
            Meta::NameStringValue { name, value } => name,
            Meta::NameIntegerValue { name, value } => name,
            Meta::NameListMeta { name, value } => name,
        }
    }

    fn to_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let meta_lit = Attribute::expand_meta_lit(&(self.meta), indent + 1);
        format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
    }
}

pub fn attributes_to_literal(attributes: &Vec<Attribute>, indent: usize) -> Vec<String> {
    let mut attributes_literal: Vec<String> = vec![];
    for attribute in attributes {
        attributes_literal.push(attribute.to_literal(indent))
    }
    attributes_literal
}
