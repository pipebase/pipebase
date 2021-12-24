use super::Map;
use crate::common::{ConfigInto, FromConfig, FromPath, Split};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StringSplitterConfig {
    pub pattern: String,
}

impl FromPath for StringSplitterConfig {}

#[async_trait]
impl ConfigInto<StringSplitter> for StringSplitterConfig {}

#[async_trait]
impl FromConfig<StringSplitterConfig> for StringSplitter {
    async fn from_config(config: StringSplitterConfig) -> anyhow::Result<Self> {
        Ok(StringSplitter {
            pattern: config.pattern,
        })
    }
}

/// Split string with pattern
pub struct StringSplitter {
    pub pattern: String,
}

impl Split<String, String, Vec<String>> for StringSplitter {
    /// Split string as a vector of strings
    fn split(s: String, pattern: &str) -> Vec<String> {
        let mut splits: Vec<String> = Vec::new();
        for item in s.split(pattern) {
            splits.push(item.to_owned())
        }
        splits
    }
}

/// # Parameters
/// * String: input
/// * Vec<String>: output
#[async_trait]
impl Map<String, Vec<String>, StringSplitterConfig> for StringSplitter {
    async fn map(&mut self, data: String) -> anyhow::Result<Vec<String>> {
        Ok(Self::split(data, &self.pattern))
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<String>, records: Vec<String>) {
        for record in records {
            let _ = tx.send(record).await;
        }
    }

    #[tokio::test]
    async fn test_string_spliter() {
        let (tx0, rx0) = channel!(String, 1024);
        let (tx1, mut rx1) = channel!(Vec<String>, 1024);
        let pipe = mapper!("text_splitter");
        let f0 = populate_records(tx0, vec!["foo bar".to_owned()]);
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            StringSplitterConfig,
            "resources/catalogs/text_splitter.yml",
            [tx1],
            rx0
        )]);
        let splitted: &[String] = &rx1.recv().await.unwrap();
        assert_eq!(2, splitted.len());
        assert_eq!("foo", splitted[0]);
        assert_eq!("bar", splitted[1]);
    }
}
