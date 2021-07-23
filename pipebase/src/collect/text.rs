use serde::Deserialize;
use tokio::time::Interval;

use super::Collect;
use crate::common::{ConfigInto, FromConfig, FromPath, Period, Render};
use async_trait::async_trait;

#[derive(Deserialize)]
pub struct TextCollectorConfig {
    flush_period: Period,
    separator: String
}

impl FromPath for TextCollectorConfig {}

impl ConfigInto<TextCollector> for TextCollectorConfig {}
pub struct TextCollector {
    buffer: String,
    flush_period: Period,
    separator: String
}

#[async_trait]
impl FromConfig<TextCollectorConfig> for TextCollector {
    async fn from_config(config: TextCollectorConfig) -> anyhow::Result<Self> {
        Ok(TextCollector {
            buffer: String::new(),
            flush_period: config.flush_period,
            separator: config.separator
        })
    }
}

/// # Parameters
/// * T: input
/// * String: output
#[async_trait]
impl<T> Collect<T, String, TextCollectorConfig> for TextCollector 
where
    T: Render
{
    async fn collect(&mut self, t: T) {
        if !self.buffer.is_empty() {
            self.buffer.push_str(self.separator)
        }
        self.buffer.push_str(t.render())
    }

    async fn flush(&mut self) -> String {
        let buffer = self.buffer.clone();
        self.buffer.clear();
        buffer
    }

    fn get_flush_interval(&self) -> Interval {
        let flush_period = self.flush_period.clone();
        tokio::time::interval(flush_period.into())
    }
}