use std::usize;

use crate::api::utils::indent_literal;
use crate::api::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::meta::{attributes_to_literal, Attribute};

#[derive(Clone, Display, EnumString, Debug, Deserialize)]
pub enum BaseType {
    #[strum(to_string = "bool")]
    Boolean,
    #[strum(to_string = "char")]
    Character,
    #[strum(to_string = "String")]
    String,
    #[strum(to_string = "i8")]
    Byte,
    #[strum(to_string = "u8")]
    UnsignedByte,
    #[strum(to_string = "i16")]
    Short,
    #[strum(to_string = "u16")]
    UnsignedShort,
    #[strum(to_string = "i32")]
    Integer,
    #[strum(to_string = "u32")]
    UnsignedInteger,
    #[strum(to_string = "isize")]
    Size,
    #[strum(to_string = "usize")]
    UnsignedSize,
    #[strum(to_string = "i64")]
    Long,
    #[strum(to_string = "u64")]
    UnsignedLong,
    #[strum(to_string = "i128")]
    LongLong,
    #[strum(to_string = "u128")]
    UnsignedLongLong,
    #[strum(to_string = "f32")]
    Float,
    #[strum(to_string = "f64")]
    Double,
    #[strum(to_string = "object")]
    Object { name: String },
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataType {
    // named data type - object's field
    // snake case validation
    pub name: Option<String>,
    pub base_type: BaseType,
    pub attributes: Option<Vec<Attribute>>,
    pub is_optional: Option<bool>,
    pub is_scalar: Option<bool>,
    pub size: Option<usize>,
}

impl DataType {
    pub fn get_data_type_literal(&self, indent: usize) -> String {
        let ty_lit = match self.base_type.to_owned() {
            BaseType::Object { name } => name,
            ty => ty.to_string(),
        };
        let ty_lit = match self.is_scalar {
            Some(is_scalar) => match is_scalar {
                true => ty_lit,
                false => match self.size {
                    Some(size) => format!("[{}; {}]", ty_lit, size),
                    None => format!("Vec<{}>", ty_lit),
                },
            },
            None => ty_lit,
        };
        let ty_lit = match self.is_optional {
            Some(is_optional) => match is_optional {
                true => format!("Option<{}>", ty_lit),
                false => ty_lit,
            },
            None => ty_lit,
        };
        let indent_lit = indent_literal(indent);
        format!("{}{}", indent_lit, ty_lit)
    }

    pub fn get_attributes_literal(&self, indent: usize) -> Vec<String> {
        let attributes = match self.attributes.to_owned() {
            Some(attributes) => attributes,
            None => return vec![],
        };
        attributes_to_literal(&attributes, indent)
    }
}

impl Entity for DataType {
    fn get_name(&self) -> String {
        self.name.to_owned().unwrap()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.base_type.to_owned() {
            BaseType::Object { name } => vec![name],
            _ => vec![],
        }
    }

    // get named field literal
    fn to_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let attributes_lit = self.get_attributes_literal(indent).join("\n");
        let literal = format!(
            "{}pub {}: {}",
            indent_lit,
            self.name.to_owned().unwrap(),
            self.get_data_type_literal(0)
        );
        match attributes_lit.is_empty() {
            true => literal,
            false => format!("{}\n{}", attributes_lit, literal),
        }
    }
}

impl<V: VisitEntity<DataType>> EntityAccept<V> for DataType {}

#[derive(Debug, Deserialize)]
pub struct Object {
    // TODO: (Camel Case Validation)
    pub name: String,
    pub trait_derives: Option<Vec<String>>,
    pub attributes: Option<Vec<Attribute>>,
    pub fields: Vec<DataType>,
}

impl Object {
    pub fn get_trait_derives_literal(&self, indent: usize) -> Option<String> {
        let trait_derives = match self.trait_derives.to_owned() {
            Some(trait_derives) => trait_derives,
            None => return None,
        };
        let trait_derives_lit = trait_derives.join(", ");
        let indent_lit = indent_literal(indent);
        Some(format!("{}#[derive({})]", indent_lit, trait_derives_lit))
    }

    pub fn get_attributes_literal(&self, indent: usize) -> Vec<String> {
        let attributes = match self.attributes.to_owned() {
            Some(attributes) => attributes,
            None => return vec![],
        };
        attributes_to_literal(&attributes, indent)
    }
}

impl Entity for Object {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        let mut dep: Vec<String> = vec![];
        for field in self.fields.as_slice() {
            dep.extend(field.list_dependency())
        }
        dep
    }

    fn to_literal(&self, indent: usize) -> String {
        let mut field_lits: Vec<String> = vec![];
        for field in self.fields.as_slice() {
            field_lits.push(field.to_literal(indent + 1))
        }
        let field_lits = field_lits.join(",\n");
        let indent_lit = indent_literal(indent);
        let struct_lit = format!(
            "{}pub struct {} {{\n{}\n{}}}",
            indent_lit, self.name, field_lits, indent_lit
        );
        // derive -> attribute -> struct
        let annotation = match self.get_trait_derives_literal(indent) {
            Some(trait_derives_literal) => trait_derives_literal,
            None => return struct_lit,
        };
        let attributes_lit = self.get_attributes_literal(indent).join("\n");
        let annotation = match attributes_lit.is_empty() {
            true => annotation,
            false => format!("{}\n{}", annotation, attributes_lit),
        };
        format!("{}\n{}", annotation, struct_lit)
    }
}

impl<V: VisitEntity<Object>> EntityAccept<V> for Object {}
