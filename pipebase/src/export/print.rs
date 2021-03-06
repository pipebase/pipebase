use super::Export;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::Debug;
use std::path::Path;

#[derive(Deserialize)]
pub struct PrinterConfig {}

#[async_trait]
impl FromPath for PrinterConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(PrinterConfig {})
    }
}

impl ConfigInto<Printer> for PrinterConfig {}

/// Stdout data
pub struct Printer {}

#[async_trait]
impl FromConfig<PrinterConfig> for Printer {
    async fn from_config(_config: PrinterConfig) -> anyhow::Result<Self> {
        Ok(Printer {})
    }
}

/// # Parameters
/// * T: input
#[async_trait]
impl<T> Export<T, PrinterConfig> for Printer
where
    T: Send + Sync + Debug + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        println!("{:?}", t);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;

    #[tokio::test]
    async fn test_printer() {
        let (tx, rx) = channel!(u128, 10);
        let channels0 = pipe_channels!([tx]);
        let channels1 = pipe_channels!(rx);
        let config0 = config!(TimerConfig, "resources/catalogs/timer.yml");
        let config1 = config!(PrinterConfig);
        let timer = poller!("timer");
        let printer = exporter!("printer");
        join_pipes!([
            run_pipe!(timer, config0, channels0),
            run_pipe!(printer, config1, channels1)
        ]);
    }
}
