use super::HandleError;
use crate::common::{ConfigInto, FromConfig, FromPath, PipeError};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PipeErrorPrinterConfig {}

#[async_trait]
impl FromPath for PipeErrorPrinterConfig {
    async fn from_path<P>(_: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(PipeErrorPrinterConfig {})
    }
}

impl ConfigInto<PipeErrorPrinter> for PipeErrorPrinterConfig {}

pub struct PipeErrorPrinter {}

#[async_trait]
impl FromConfig<PipeErrorPrinterConfig> for PipeErrorPrinter {
    async fn from_config(_: PipeErrorPrinterConfig) -> anyhow::Result<Self> {
        Ok(PipeErrorPrinter {})
    }
}

#[async_trait]
impl HandleError<PipeErrorPrinterConfig> for PipeErrorPrinter {
    async fn handle_error(&mut self, pipe_error: PipeError) -> anyhow::Result<()> {
        println!(
            "[Error] pipe: '{}', details: '{:#?}'",
            pipe_error.pipe_name, pipe_error.error
        );
        Ok(())
    }
}
