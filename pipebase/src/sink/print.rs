use crate::{ConfigInto, Export, FromConfig, FromFile};
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize)]
pub struct PrinterConfig {}

impl FromFile for PrinterConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(PrinterConfig {})
    }
}

impl ConfigInto<Printer> for PrinterConfig {}

pub struct Printer {}

#[async_trait]
impl FromConfig<PrinterConfig> for Printer {
    async fn from_config(
        _config: &PrinterConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Printer {})
    }
}

#[async_trait]
impl<T: Send + Sync + Display + 'static> Export<T, PrinterConfig> for Printer {
    async fn export(
        &mut self,
        t: &T,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
