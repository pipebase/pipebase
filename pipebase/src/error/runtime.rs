use super::HandleError;
use crate::common::{ConfigInto, PipeError, Result};
use tokio::sync::mpsc::Receiver;

pub struct ErrorHandler {}

impl ErrorHandler {
    pub async fn run<H, C>(&mut self, config: C, mut rx: Receiver<PipeError>) -> Result<()>
    where
        C: ConfigInto<H> + Send,
        H: HandleError<C>,
    {
        let mut handler = config.config_into().await?;
        loop {
            let pipe_error = match rx.recv().await {
                Some(pipe_error) => pipe_error,
                None => break,
            };
            match handler.handle_error(pipe_error).await {
                Ok(_) => continue,
                Err(e) => {
                    log::error!("error handler error '{:#?}'", e)
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
    ($error_handler:ident, $config:ty, $rx:ident) => {
        run_error_handler!($error_handler, $config, "", $rx)
    };
    ($error_handler:ident, $config:ty, $path:expr, $rx:ident) => {{
        tokio::spawn(async move {
            let config = <$config>::from_path($path)
                .await
                .expect(&format!("invalid config file location {}", $path));
            match $error_handler.run(config, $rx).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    log::error!("error handler exit with error {:#?}", err);
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
