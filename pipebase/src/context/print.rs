use super::StoreContext;
use crate::common::{ConfigInto, Context, FromConfig, FromPath, Period, PipeContext, State};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{sleep, Interval};

#[derive(Deserialize)]
pub struct ContextPrinterConfig {
    pub interval: Period,
    pub delay: Option<Period>,
}

impl FromPath for ContextPrinterConfig {}

#[async_trait]
impl ConfigInto<ContextPrinter> for ContextPrinterConfig {}

pub struct ContextPrinter {
    interval: Interval,
    delay: Duration,
    contexts: HashMap<String, std::sync::Arc<Context>>,
}

#[async_trait]
impl FromConfig<ContextPrinterConfig> for ContextPrinter {
    async fn from_config(config: ContextPrinterConfig) -> anyhow::Result<Self> {
        let delay = match config.delay {
            Some(period) => period.into(),
            None => Duration::from_micros(0),
        };
        Ok(ContextPrinter {
            interval: tokio::time::interval(config.interval.into()),
            delay: delay,
            contexts: HashMap::new(),
        })
    }
}

#[async_trait]
impl StoreContext<ContextPrinterConfig> for ContextPrinter {
    fn store_context(&mut self, pipe_name: String, context: std::sync::Arc<Context>) {
        self.contexts.insert(pipe_name, context);
    }

    fn load_context(&self, pipe_name: &str) -> Option<&std::sync::Arc<Context>> {
        self.contexts.get(pipe_name)
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        sleep(self.delay).await;
        loop {
            self.interval.tick().await;
            let mut done: usize = 0;
            for (pipe_name, ctx) in &self.contexts {
                let ref state = ctx.get_state();
                let total_run = ctx.get_total_run();
                let failure_run = ctx.get_failure_run();
                let display = PipeContext::new(
                    pipe_name.to_owned(),
                    state.to_owned(),
                    total_run,
                    failure_run,
                );
                print!("{}", display);
                if state == &State::Done {
                    done += 1;
                }
            }
            if done == self.contexts.len() {
                log::info!("all pipe in Done state, exit context printer");
                break;
            }
        }
        Ok(())
    }
}
