use async_trait::async_trait;
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use super::Export;
use crate::common::{
    send_pipe_error, ConfigInto, Context, HasContext, Pipe, PipeError, Result, State,
    SubscribeError,
};

pub struct Exporter<'a> {
    name: &'a str,
    context: Arc<Context>,
    etx: Option<Sender<PipeError>>,
}

/// Start loop
/// * Receive data from upstream
/// * Export to external
/// # Parameters
/// * T: input
/// * E: exporter
#[async_trait]
impl<'a, T, E, C> Pipe<T, (), E, C> for Exporter<'a>
where
    T: Send + 'static,
    E: Export<T, C> + 'static,
    C: ConfigInto<E> + Send + Sync + 'static,
{
    async fn run(
        &mut self,
        config: C,
        txs: Vec<Sender<()>>,
        mut rx: Option<Receiver<T>>,
    ) -> Result<()> {
        assert!(rx.is_some(), "sink {} has no upstreams", self.name);
        assert!(txs.is_empty(), "sink {} has invalid downstreams", self.name);
        let mut exporter = config.config_into().await?;
        let rx = rx.as_mut().unwrap();
        log::info!("exporter {} run ...", self.name);
        loop {
            self.context.set_state(State::Receive);
            let t = match rx.recv().await {
                Some(t) => t,
                None => {
                    break;
                }
            };
            self.context.set_state(State::Export);
            match exporter.export(t).await {
                Ok(_) => (),
                Err(err) => {
                    error!("exporter {} error '{:#?}'", self.name, err);
                    self.context.inc_failure_run();
                    send_pipe_error(self.etx.as_ref(), PipeError::new(self.name.to_owned(), err))
                        .await
                }
            };
            self.context.inc_total_run();
        }
        log::info!("exporter {} exit ...", self.name);
        self.context.set_state(State::Done);
        Ok(())
    }
}

impl<'a> HasContext for Exporter<'a> {
    fn get_name(&self) -> String {
        self.name.to_owned()
    }

    fn get_context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

impl<'a> Exporter<'a> {
    pub fn new(name: &'a str) -> Self {
        Exporter {
            name,
            context: Default::default(),
            etx: None,
        }
    }
}

impl<'a> SubscribeError for Exporter<'a> {
    fn subscribe_error(&mut self, tx: Sender<crate::common::PipeError>) {
        self.etx = Some(tx)
    }
}

#[macro_export]
macro_rules! exporter {
    (
        $name:expr
    ) => {{
        Exporter::new($name)
    }};
}
