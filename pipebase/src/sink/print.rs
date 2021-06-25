use crate::{ConfigInto, Export, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::Display;
use std::path::Path;

#[derive(Deserialize)]
pub struct PrinterConfig {}

impl FromPath for PrinterConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(PrinterConfig {})
    }
}

impl ConfigInto<Printer> for PrinterConfig {}

pub struct Printer {}

#[async_trait]
impl FromConfig<PrinterConfig> for Printer {
    async fn from_config(_config: &PrinterConfig) -> anyhow::Result<Self> {
        Ok(Printer {})
    }
}

#[async_trait]
impl<T> Export<T, PrinterConfig> for Printer
where
    T: Send + Sync + Display + 'static,
{
    async fn export(&mut self, t: &T) -> anyhow::Result<()> {
        println!("{}", t);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[tokio::test]
    async fn test_printer() {
        let (tx, rx) = channel!(u128, 10);
        let mut timer = poller!("timer", "resources/catalogs/timer.yml", TimerConfig, [tx]);
        let mut printer = exporter!("printer", "", PrinterConfig, rx, []);
        spawn_join!(timer, printer);
    }
}
