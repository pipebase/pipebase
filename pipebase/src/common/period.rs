use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Deserialize)]
pub enum Period {
    Millis(u64),
    Secs(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
}

impl From<Period> for Duration {
    fn from(period: Period) -> Self {
        match period {
            Period::Millis(m) => Duration::from_millis(m),
            Period::Secs(s) => Duration::from_secs(s),
            Period::Minutes(m) => Duration::from_secs(m * 60),
            Period::Hours(h) => Duration::from_secs(h * 3600),
            Period::Days(d) => Duration::from_secs(d * 3600 * 3600),
        }
    }
}
