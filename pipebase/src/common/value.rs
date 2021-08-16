use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    UnsignedInteger(u32),
    Integer(i32),
    UnsignedLong(u64),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    UnsignedBytes(Vec<u8>),
    Array(Vec<Value>),
    Attributes(HashMap<String, Value>),
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::UnsignedInteger(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::UnsignedLong(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Long(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::UnsignedBytes(v)
    }
}

impl<V> From<Vec<V>> for Value
where
    V: Into<Value>,
{
    fn from(v: Vec<V>) -> Self {
        let v: Vec<Value> = v.into_iter().map(|item| item.into()).collect();
        Value::Array(v)
    }
}

impl<V> From<HashMap<String, V>> for Value
where
    V: Into<Value>,
{
    fn from(v: HashMap<String, V>) -> Self {
        let v: HashMap<String, Value> = v.into_iter().map(|(k, v)| (k, v.into())).collect();
        Value::Attributes(v)
    }
}

pub trait IntoAttributes {
    fn into_attributes(self) -> HashMap<String, Value>;

    fn into_attribute_tuples(self) -> Vec<(String, Value)>;
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;
    use std::collections::HashMap;

    #[derive(IntoAttributes)]
    struct Record {
        key: String,
        value: u32,
    }

    #[test]
    fn test_record_into_attributes() {
        let r = Record {
            key: "foo".to_owned(),
            value: 1,
        };
        let attributes: HashMap<String, Value> = r.into_attributes();
        assert_eq!(
            &Value::from("foo".to_owned()),
            attributes.get("key").unwrap()
        );
        assert_eq!(&Value::from(1 as u32), attributes.get("value").unwrap());
    }

    #[derive(IntoAttributes)]
    struct RecordWithAlias {
        #[attribute(alias = "id")]
        key: String,
        value: u32,
    }

    #[test]
    fn test_record_with_alias_into_attributes() {
        let r = RecordWithAlias {
            key: "foo".to_owned(),
            value: 1,
        };
        let attributes: HashMap<String, Value> = r.into_attributes();
        assert_eq!(
            &Value::from("foo".to_owned()),
            attributes.get("id").unwrap()
        );
        assert_eq!(&Value::from(1 as u32), attributes.get("value").unwrap());
    }
}
