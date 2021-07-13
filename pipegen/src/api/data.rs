use std::usize;

use crate::api::meta::metas_to_display;
use crate::api::utils::indent_literal;
use crate::api::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use std::fmt::Display;

use super::meta::{metas_to_literal, Meta};

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
    Count32,
    Averagef32,
    Object(String),
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
    Pair {
        left_data_ty: Box<DataType>,
        right_data_ty: Box<DataType>,
    },
}

pub fn data_ty_to_literal(ty: &DataType) -> String {
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
        DataType::Count32 => "pipebase::Count32".to_owned(),
        DataType::Averagef32 => "pipebase::Averagef32".to_owned(),
        DataType::Object(object) => object.to_owned(),
        DataType::Vec { data_ty } => {
            let data_ty_lit = data_ty_to_literal(data_ty);
            format!("std::vec::Vec<{}>", data_ty_lit)
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
            format!(
                "std::collections::HashMap<{}, {}>",
                key_data_ty, value_data_ty
            )
        }
        DataType::HashSet { data_ty } => {
            let data_ty_lit = data_ty_to_literal(data_ty);
            format!("std::collections::HashSet<{}>", data_ty_lit)
        }
        DataType::Pair {
            left_data_ty,
            right_data_ty,
        } => {
            let left_data_ty = data_ty_to_literal(left_data_ty);
            let right_data_ty = data_ty_to_literal(right_data_ty);
            format!("pipebase::Pair<{}, {}>", left_data_ty, right_data_ty)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataField {
    // either named or unamed data field
    name: Option<String>,
    data_ty: DataType,
    metas: Option<Vec<Meta>>,
    is_boxed: Option<bool>,
    is_optional: Option<bool>,
    is_public: Option<bool>,
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

    pub fn get_metas_literal(&self, indent: usize) -> Option<String> {
        let metas = match self.metas.to_owned() {
            Some(metas) => metas,
            None => return None,
        };
        Some(metas_to_literal(&metas, indent))
    }

    pub fn get_pub_literal(&self) -> String {
        let is_public = match self.is_public {
            Some(is_public) => is_public,
            None => return "pub ".to_owned(),
        };
        match is_public {
            true => "pub ".to_owned(),
            false => "".to_owned(),
        }
    }

    pub fn display(&self) -> (String, String, String) {
        let name_display = match self.name {
            Some(ref name) => name.to_owned(),
            None => String::new(),
        };
        let type_display = self.get_data_type_literal(0);
        let metas_display = match self.metas {
            Some(ref metas) => metas_to_display(metas),
            None => String::new(),
        };
        (name_display, type_display, metas_display)
    }
}

impl Entity for DataField {
    fn get_id(&self) -> String {
        match self.name {
            Some(ref name) => name.to_owned(),
            None => String::new(),
        }
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.data_ty.to_owned() {
            DataType::Object(object) => vec![object],
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
        let metas_lit = self.get_metas_literal(indent);
        let pub_lit = self.get_pub_literal();
        let literal = format!(
            "{}{}{}: {}",
            indent_lit,
            pub_lit,
            name,
            self.get_data_type_literal(0)
        );
        match metas_lit {
            None => literal,
            Some(metas_lit) => format!("{}\n{}", metas_lit, literal),
        }
    }
}

impl<V: VisitEntity<DataField>> EntityAccept<V> for DataField {}

#[derive(Debug, Deserialize, Clone)]
pub struct Object {
    ty: String,
    metas: Option<Vec<Meta>>,
    fields: Vec<DataField>,
}

impl Object {
    pub fn new(ty: String, metas: Vec<Meta>, fields: Vec<DataField>) -> Self {
        Object {
            ty: ty,
            metas: Some(metas),
            fields: fields,
        }
    }

    pub fn get_metas_literal(&self, indent: usize) -> Option<String> {
        let metas = match self.metas.to_owned() {
            Some(metas) => metas,
            None => return None,
        };
        Some(metas_to_literal(&metas, indent))
    }

    pub fn get_fields(&self) -> &Vec<DataField> {
        &self.fields
    }
}

impl Entity for Object {
    fn get_id(&self) -> String {
        self.ty.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        self.fields
            .iter()
            .map(|field| field.list_dependency())
            .flatten()
            .collect()
    }

    fn to_literal(&self, indent: usize) -> String {
        let field_lits: Vec<String> = self
            .fields
            .iter()
            .map(|field| field.to_literal(indent + 1))
            .collect();
        let field_lits = field_lits.join(",\n");
        let indent_lit = indent_literal(indent);
        let struct_lit = format!(
            "{}pub struct {} {{\n{}\n{}}}",
            indent_lit, &self.ty, field_lits, indent_lit
        );
        let metas_lit = self.get_metas_literal(indent);
        let metas_lit = match metas_lit {
            None => return struct_lit,
            Some(metas_lit) => metas_lit,
        };
        format!("{}\n{}", metas_lit, struct_lit)
    }
}

impl<V: VisitEntity<Object>> EntityAccept<V> for Object {}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Type: {}", self.ty)?;
        match self.metas {
            Some(ref metas) => {
                writeln!(f, "Type Metas: {}", metas_to_display(metas))?;
            }
            None => (),
        };
        writeln!(f, "Fields:")?;
        let mut displays: Vec<(String, String, String)> = Vec::new();
        displays.push((
            OBJECY_DISPLAY_HEADER_NAME.to_owned(),
            OBJECY_DISPLAY_HEADER_TYPE.to_owned(),
            OBJECY_DISPLAY_HEADER_METAS.to_owned(),
        ));
        let mut name_width = OBJECY_DISPLAY_HEADER_NAME.len();
        let mut ty_width = OBJECY_DISPLAY_HEADER_TYPE.len();
        let mut metas_width = OBJECY_DISPLAY_HEADER_METAS.len();
        for field in &self.fields {
            let (name, ty, metas) = field.display();
            name_width = name_width.max(name.len());
            ty_width = ty_width.max(ty.len());
            metas_width = metas_width.max(metas.len());
            displays.push((name, ty, metas));
        }
        // row of fields
        for (name, ty, metas) in displays {
            writeln!(
                f,
                "{name:>nw$} {ty:>tw$} {metas:>mw$}",
                name = name,
                nw = name_width,
                ty = ty,
                tw = ty_width,
                metas = metas,
                mw = metas_width
            )?;
        }
        write!(f, "")
    }
}

const OBJECY_DISPLAY_HEADER_NAME: &str = "Name";
const OBJECY_DISPLAY_HEADER_TYPE: &str = "Type";
const OBJECY_DISPLAY_HEADER_METAS: &str = "Metas";
