use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use std::collections::HashMap;

use super::Period;

#[derive(Debug, Clone, PartialEq)]
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
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Duration(Duration),
    LocalTime(DateTime<Local>),
    UtcTime(DateTime<Utc>),
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

impl From<NaiveDate> for Value {
    fn from(v: NaiveDate) -> Self {
        Value::Date(v)
    }
}

impl From<NaiveDateTime> for Value {
    fn from(v: NaiveDateTime) -> Self {
        Value::DateTime(v)
    }
}

impl From<Duration> for Value {
    fn from(v: Duration) -> Self {
        Value::Duration(v)
    }
}

impl From<Period> for Value {
    fn from(v: Period) -> Self {
        let v = match v {
            Period::Days(v) => Duration::days(v),
            Period::Hours(v) => Duration::hours(v),
            Period::Minutes(v) => Duration::minutes(v),
            Period::Secs(v) => Duration::seconds(v),
            Period::Millis(v) => Duration::milliseconds(v),
        };
        Value::Duration(v)
    }
}

impl From<DateTime<Local>> for Value {
    fn from(v: DateTime<Local>) -> Self {
        Value::LocalTime(v)
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(v: DateTime<Utc>) -> Self {
        Value::UtcTime(v)
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
