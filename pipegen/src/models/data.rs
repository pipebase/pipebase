use std::usize;

use crate::models::meta::metas_to_display;
use crate::models::utils::indent_literal;
use crate::models::{Entity, EntityAccept, VisitEntity};
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
    PathBuf,
    Count32,
    Averagef32,
    Object(String),
    Booleans,
    Characters,
    Strings,
    Bytes,
    UnsignedBytes,
    Shorts,
    UnsignedShorts,
    Integers,
    UnsignedIntegers,
    Sizes,
    UnsignedSizes,
    Longs,
    UnsignedLongs,
    LongLongs,
    UnsignedLongLongs,
    Floats,
    Doubles,
    PathBufs,
    Count32s,
    Averagef32s,
    Objects(String),
    Vec {
        ty: Box<DataType>,
    },
    Array {
        ty: Box<DataType>,
        size: usize,
    },
    Tuple {
        tys: Vec<DataType>,
    },
    HashMap {
        kty: Box<DataType>,
        vty: Box<DataType>,
    },
    HashSet {
        ty: Box<DataType>,
    },
    Pair {
        lty: Box<DataType>,
        rty: Box<DataType>,
    },
    Tuples {
        tys: Vec<DataType>,
    },
    Pairs {
        lty: Box<DataType>,
        rty: Box<DataType>,
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
        DataType::PathBuf => "std::path::PathBuf".to_owned(),
        DataType::Count32 => "Count32".to_owned(),
        DataType::Averagef32 => "Averagef32".to_owned(),
        DataType::Object(object) => object.to_owned(),
        DataType::Booleans => {
            let ty_lit = data_ty_to_literal(&DataType::Boolean);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Characters => {
            let ty_lit = data_ty_to_literal(&DataType::Character);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Strings => {
            let ty_lit = data_ty_to_literal(&DataType::String);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Bytes => {
            let ty_lit = data_ty_to_literal(&DataType::Byte);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedBytes => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedByte);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Shorts => {
            let ty_lit = data_ty_to_literal(&DataType::Short);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedShorts => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedShort);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Integers => {
            let ty_lit = data_ty_to_literal(&DataType::Integer);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedIntegers => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedInteger);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Sizes => {
            let ty_lit = data_ty_to_literal(&DataType::Size);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedSizes => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedSize);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Longs => {
            let ty_lit = data_ty_to_literal(&DataType::Long);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedLongs => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedLong);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::LongLongs => {
            let ty_lit = data_ty_to_literal(&DataType::LongLong);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::UnsignedLongLongs => {
            let ty_lit = data_ty_to_literal(&DataType::UnsignedLongLong);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Floats => {
            let ty_lit = data_ty_to_literal(&DataType::Float);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Doubles => {
            let ty_lit = data_ty_to_literal(&DataType::Double);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::PathBufs => {
            let ty_lit = data_ty_to_literal(&DataType::PathBuf);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Count32s => {
            let ty_lit = data_ty_to_literal(&DataType::Count32);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Averagef32s => {
            let ty_lit = data_ty_to_literal(&DataType::Averagef32);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Objects(object) => {
            let ty_lit = data_ty_to_literal(&DataType::Object(object.to_owned()));
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Vec { ty } => {
            let ty_lit = data_ty_to_literal(ty);
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Array { ty, size } => {
            let ty_lit = data_ty_to_literal(ty);
            format!("[{}; {}]", ty_lit, size)
        }
        DataType::Tuple { tys } => {
            let tys: Vec<String> = tys.iter().map(|ty| data_ty_to_literal(ty)).collect();
            format!("({})", tys.join(", "))
        }
        DataType::HashMap { kty, vty } => {
            let kty = data_ty_to_literal(kty);
            let vty = data_ty_to_literal(vty);
            format!("std::collections::HashMap<{}, {}>", kty, vty)
        }
        DataType::HashSet { ty } => {
            let ty_lit = data_ty_to_literal(ty);
            format!("std::collections::HashSet<{}>", ty_lit)
        }
        DataType::Pair { lty, rty } => {
            let lty = data_ty_to_literal(lty);
            let rty = data_ty_to_literal(rty);
            format!("Pair<{}, {}>", lty, rty)
        }
        DataType::Tuples { tys } => {
            let ty_lit = data_ty_to_literal(&DataType::Tuple {
                tys: tys.to_owned(),
            });
            format!("std::vec::Vec<{}>", ty_lit)
        }
        DataType::Pairs { lty, rty } => {
            let ty_lit = data_ty_to_literal(&DataType::Pair {
                lty: lty.to_owned(),
                rty: rty.to_owned(),
            });
            format!("std::vec::Vec<{}>", ty_lit)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataField {
    // either named or unamed data field
    name: Option<String>,
    ty: DataType,
    metas: Option<Vec<Meta>>,
    is_boxed: Option<bool>,
    is_optional: Option<bool>,
    is_public: Option<bool>,
}

impl DataField {
    pub fn init(&mut self) {
        if self.is_boxed.is_none() {
            self.is_boxed = Some(false);
        }
        if self.is_optional.is_none() {
            self.is_optional = Some(false);
        }
        if self.is_public.is_none() {
            self.is_public = Some(true);
        }
        if self.metas.is_none() {
            self.metas = Some(Vec::new())
        }
    }

    pub fn get_data_type_literal(&self, indent: usize) -> String {
        let ty_lit = data_ty_to_literal(&self.ty);
        let ty_lit = match self.is_boxed.expect("is_boxed not inited") {
            true => format!("Box<{}>", ty_lit),
            false => ty_lit,
        };
        let ty_lit = match self.is_optional.expect("is_optional not inited") {
            true => format!("Option<{}>", ty_lit),
            false => ty_lit,
        };
        let indent_lit = indent_literal(indent);
        format!("{}{}", indent_lit, ty_lit)
    }

    pub fn get_metas_literal(&self, indent: usize) -> Option<String> {
        let metas = self.metas.as_ref().expect("field metas not inited");
        Some(metas_to_literal(metas, indent))
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
        match self.ty.to_owned() {
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
            fields: fields
                .into_iter()
                .map(|mut f| {
                    f.init();
                    f
                })
                .collect(),
        }
    }

    pub fn init(&mut self) {
        if self.metas.is_none() {
            self.metas = Some(Vec::new())
        }
        for field in self.fields.as_mut_slice() {
            field.init();
        }
    }

    pub fn get_metas_literal(&self, indent: usize) -> Option<String> {
        let metas = self.metas.as_ref().expect("object metas not inited");
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
