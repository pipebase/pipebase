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
