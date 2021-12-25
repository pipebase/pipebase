use super::HandleError;
use crate::common::{ConfigInto, PipeError, Result};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};

pub struct ErrorHandler {}

impl ErrorHandler {
    pub async fn run<H, C>(self, config: C, mut rx: Receiver<PipeError>) -> Result<()>
    where
        C: ConfigInto<H> + Send,
        H: HandleError<C>,
    {
        let mut handler = config.config_into().await?;
        info!("error handler run ...");
        while let Some(pipe_error) = rx.recv().await {
            match handler.handle_error(pipe_error).await {
                Ok(_) => continue,
                Err(e) => {
                    error!("error handler error '{:#?}'", e)
                }
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! error_handler {
    () => {{
        ErrorHandler {}
    }};
}

#[macro_export]
macro_rules! run_error_handler {
    ($error_handler:ident, $config:ident, $rx:ident) => {{
        tokio::spawn(async move {
            match $error_handler.run($config, $rx).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    tracing::error!("error handler exit with error {:#?}", err);
                    Err(err)
                }
            }
        })
    }};
}

#[macro_export]
macro_rules! subscribe_error_handler {
    ([$( $pipe:expr ), *], $tx:ident) => {
        {
            $(
                $pipe.subscribe_error($tx.clone());
            )*
        }
    };
}
