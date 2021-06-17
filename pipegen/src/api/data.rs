use std::usize;

use crate::api::utils::indent_literal;
use crate::api::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;

use super::meta::{attributes_to_literal, Attribute};

#[derive(Clone, Debug, Deserialize)]
pub enum DataType {
    Boolean,
    Character,
    String,
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Integer,
    UnsignedInteger,
    Size,
    UnsignedSize,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
    Float,
    Double,
    Object {
        ty: String,
    },
    Vec {
        data_ty: Box<DataType>,
    },
    Array {
        data_ty: Box<DataType>,
        size: usize,
    },
    Tuple {
        data_tys: Vec<DataType>,
    },
    HashMap {
        key_data_ty: Box<DataType>,
        value_data_ty: Box<DataType>,
    },
    HashSet {
        data_ty: Box<DataType>,
    },
}

fn data_ty_to_literal(ty: &DataType) -> String {
    match ty {
        DataType::Boolean => "bool".to_owned(),
        DataType::Character => "char".to_owned(),
        DataType::String => "String".to_owned(),
        DataType::Byte => "i8".to_owned(),
        DataType::UnsignedByte => "u8".to_owned(),
        DataType::Short => "i16".to_owned(),
        DataType::UnsignedShort => "u16".to_owned(),
        DataType::Integer => "i32".to_owned(),
        DataType::UnsignedInteger => "u32".to_owned(),
        DataType::Size => "isize".to_owned(),
        DataType::UnsignedSize => "usize".to_owned(),
        DataType::Long => "i64".to_owned(),
        DataType::UnsignedLong => "u64".to_owned(),
        DataType::LongLong => "i128".to_owned(),
        DataType::UnsignedLongLong => "u128".to_owned(),
        DataType::Float => "f32".to_owned(),
        DataType::Double => "f64".to_owned(),
        DataType::Object { ty } => ty.to_owned(),
        DataType::Vec { data_ty } => {
            let data_ty_lit = data_ty_to_literal(data_ty);
            format!("Vec<{}>", data_ty_lit)
        }
        DataType::Array { data_ty, size } => {
            let data_ty_lit = data_ty_to_literal(data_ty);
            format!("[{}; {}]", data_ty_lit, size)
        }
        DataType::Tuple { data_tys } => {
            let data_tys: Vec<String> = data_tys
                .iter()
                .map(|data_ty| data_ty_to_literal(data_ty))
                .collect();
            format!("({})", data_tys.join(", "))
        }
        DataType::HashMap {
            key_data_ty,
            value_data_ty,
        } => {
            let key_data_ty = data_ty_to_literal(key_data_ty);
            let value_data_ty = data_ty_to_literal(value_data_ty);
            format!("HashMap<{}, {}>", key_data_ty, value_data_ty)
        }
        DataType::HashSet { data_ty } => {
            let data_ty_lit = data_ty_to_literal(data_ty);
            format!("HashSet<{}>", data_ty_lit)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataField {
    // either named or unamed data field
    pub name: Option<String>,
    pub data_ty: DataType,
    pub attributes: Option<Vec<Attribute>>,
    pub is_boxed: Option<bool>,
    pub is_optional: Option<bool>,
}

impl DataField {
    pub fn get_data_type_literal(&self, indent: usize) -> String {
        let ty_lit = data_ty_to_literal(&self.data_ty);
        let ty_lit = match self.is_boxed {
            Some(is_boxed) => match is_boxed {
                true => format!("Box<{}>", ty_lit),
                false => ty_lit,
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

    pub fn get_data_type(&self) -> DataType {
        self.data_ty.to_owned()
    }
}

impl Entity for DataField {
    fn get_name(&self) -> String {
        match self.name {
            Some(ref name) => name.to_owned(),
            None => String::new(),
        }
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.data_ty.to_owned() {
            DataType::Object { ty } => vec![ty],
            _ => vec![],
        }
    }

    // get named field literal
    fn to_literal(&self, indent: usize) -> String {
        // if no name data type, return type literal only
        let name = match self.name.to_owned() {
            Some(name) => name,
            None => return self.get_data_type_literal(indent),
        };
        let indent_lit = indent_literal(indent);
        let attributes_lit = self.get_attributes_literal(indent).join("\n");
        let literal = format!(
            "{}pub {}: {}",
            indent_lit,
            name,
            self.get_data_type_literal(0)
        );
        match attributes_lit.is_empty() {
            true => literal,
            false => format!("{}\n{}", attributes_lit, literal),
        }
    }
}

impl<V: VisitEntity<DataField>> EntityAccept<V> for DataField {}

#[derive(Debug, Deserialize, Clone)]
pub struct Object {
    // TODO: (Camel Case Validation)
    pub ty: String,
    pub traits: Option<Vec<String>>,
    pub attributes: Option<Vec<Attribute>>,
    pub fields: Vec<DataField>,
}

impl Object {
    pub fn get_trait_derives_literal(&self, indent: usize) -> Option<String> {
        let trait_derives = match self.traits.to_owned() {
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
        self.ty.to_owned()
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
            indent_lit, &self.ty, field_lits, indent_lit
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
