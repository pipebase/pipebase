use crate::api::utils::indent_literal;
use crate::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum FieldType {
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
    Structure {
        name: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
    pub is_optional: bool,
    pub is_scalar: bool,
    pub size: Option<usize>,
}

impl Field {
    pub fn get_type_literal(&self) -> String {
        let ty_lit = match self.ty.to_owned() {
            FieldType::Structure { name } => name,
            ty => ty.to_string(),
        };
        let ty_lit = match self.is_scalar {
            true => match self.size {
                Some(size) => format!("[{}; {}]", ty_lit, size),
                None => format!("Vec<{}>", ty_lit),
            },
            false => ty_lit,
        };
        let ty_lit = match self.is_optional {
            true => format!("Option<{}>", ty_lit),
            false => ty_lit,
        };
        ty_lit
    }
}

impl Entity for Field {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.ty.to_owned() {
            FieldType::Structure { name } => vec![name],
            _ => vec![],
        }
    }

    fn to_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        format!(
            "{}pub {}: {}",
            indent_lit,
            self.name,
            self.get_type_literal()
        )
    }
}

impl<V: VisitEntity<Field>> EntityAccept<V> for Field {}

#[derive(Debug, Deserialize)]
pub struct Structure {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Entity for Structure {
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
        let lit = format!(
            "{}pub struct {} {{\n{}\n{}}}",
            indent_lit, self.name, field_lits, indent_lit
        );
        lit
    }
}

impl<V: VisitEntity<Structure>> EntityAccept<V> for Structure {}
