use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Deserialize)]
pub enum Period {
    Millis(i64),
    Secs(i64),
    Minutes(i64),
    Hours(i64),
    Days(i64),
}

impl From<Period> for Duration {
    fn from(period: Period) -> Self {
        match period {
            Period::Millis(m) => Duration::from_millis(m as u64),
            Period::Secs(s) => Duration::from_secs(s as u64),
            Period::Minutes(m) => Duration::from_secs((m as u64) * 60),
            Period::Hours(h) => Duration::from_secs((h as u64) * 3600),
            Period::Days(d) => Duration::from_secs((d as u64) * 3600 * 3600),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Timestamp {
    Millis(u64),
    Secs(u64),
}

pub mod date_time_without_timezone {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date_time: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date_time.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use chrono::{NaiveDate, NaiveDateTime};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Metric {
        #[serde(with = "date_time_without_timezone")]
        pub timestamp: NaiveDateTime,
    }

    #[test]
    fn test_datetime_without_timezone() {
        let json_str = r#"{
  "timestamp": "2017-02-16 21:54:30"
}"#;
        let metric: Metric = serde_json::from_str(json_str).unwrap();
        let ts = &metric.timestamp;
        assert_eq!(ts, &NaiveDate::from_ymd(2017, 2, 16).and_hms(21, 54, 30));
        let serialized = serde_json::to_string_pretty(&metric).unwrap();
        assert_eq!(json_str, serialized.as_str());
    }
}
