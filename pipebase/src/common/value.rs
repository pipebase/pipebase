use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use std::collections::HashMap;

use super::{Period, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(Option<bool>),
    UnsignedInteger(Option<u32>),
    Integer(Option<i32>),
    UnsignedLong(Option<u64>),
    Long(Option<i64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<String>),
    Date(Option<NaiveDate>),
    DateTime(Option<NaiveDateTime>),
    Duration(Option<Duration>),
    LocalTime(Option<DateTime<Local>>),
    UtcTime(Option<DateTime<Utc>>),
    UnsignedBytes(Vec<u8>),
    Array(Vec<Value>),
    Attributes(HashMap<String, Value>),
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v.into())
    }
}

impl From<Option<bool>> for Value {
    fn from(v: Option<bool>) -> Self {
        Value::Bool(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::UnsignedInteger(v.into())
    }
}

impl From<Option<u32>> for Value {
    fn from(v: Option<u32>) -> Self {
        Value::UnsignedInteger(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v.into())
    }
}

impl From<Option<i32>> for Value {
    fn from(v: Option<i32>) -> Self {
        Value::Integer(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::UnsignedLong(v.into())
    }
}

impl From<Option<u64>> for Value {
    fn from(v: Option<u64>) -> Self {
        Value::UnsignedLong(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Long(v.into())
    }
}

impl From<Option<i64>> for Value {
    fn from(v: Option<i64>) -> Self {
        Value::Long(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v.into())
    }
}

impl From<Option<String>> for Value {
    fn from(v: Option<String>) -> Self {
        Value::String(v)
    }
}

impl From<NaiveDate> for Value {
    fn from(v: NaiveDate) -> Self {
        Value::Date(v.into())
    }
}

impl From<Option<NaiveDate>> for Value {
    fn from(v: Option<NaiveDate>) -> Self {
        Value::Date(v)
    }
}

impl From<NaiveDateTime> for Value {
    fn from(v: NaiveDateTime) -> Self {
        Value::DateTime(v.into())
    }
}

impl From<Option<NaiveDateTime>> for Value {
    fn from(v: Option<NaiveDateTime>) -> Self {
        Value::DateTime(v)
    }
}

impl From<Duration> for Value {
    fn from(v: Duration) -> Self {
        Value::Duration(v.into())
    }
}

impl From<Option<Duration>> for Value {
    fn from(v: Option<Duration>) -> Self {
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
        Value::Duration(v.into())
    }
}

impl From<Option<Period>> for Value {
    fn from(v: Option<Period>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Value::Duration(None),
        }
    }
}

impl From<DateTime<Local>> for Value {
    fn from(v: DateTime<Local>) -> Self {
        Value::LocalTime(v.into())
    }
}

impl From<Option<DateTime<Local>>> for Value {
    fn from(v: Option<DateTime<Local>>) -> Self {
        Value::LocalTime(v)
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(v: DateTime<Utc>) -> Self {
        Value::UtcTime(v.into())
    }
}

impl From<Option<DateTime<Utc>>> for Value {
    fn from(v: Option<DateTime<Utc>>) -> Self {
        Value::UtcTime(v)
    }
}

impl From<Timestamp> for Value {
    fn from(v: Timestamp) -> Self {
        match v {
            Timestamp::Millis(v) => Value::Duration(Duration::milliseconds(v as i64).into()),
            Timestamp::Secs(v) => Value::Duration(Duration::seconds(v as i64).into()),
        }
    }
}

impl From<Option<Timestamp>> for Value {
    fn from(v: Option<Timestamp>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Value::Duration(None),
        }
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
