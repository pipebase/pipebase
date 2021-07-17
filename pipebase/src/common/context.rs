use crate::{ConfigInto, FromConfig, FromPath, Period, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU64, AtomicU8};
use std::time::Duration;
use tokio::time::{sleep, Interval};

use strum::{Display, EnumString};

#[derive(Clone, Display, EnumString)]

/// Pipe running state
#[derive(PartialEq, Debug)]
pub enum State {
    #[strum(to_string = "init")]
    Init = 0,
    #[strum(to_string = "receive")]
    Receive,
    #[strum(to_string = "poll")]
    Poll,
    #[strum(to_string = "process")]
    Process,
    #[strum(to_string = "send")]
    Send,
    #[strum(to_string = "export")]
    Export,
    #[strum(to_string = "done")]
    Done,
}

fn code_to_state(state_code: u8) -> State {
    let state = match state_code {
        0 => State::Init,
        1 => State::Receive,
        2 => State::Poll,
        3 => State::Process,
        4 => State::Send,
        5 => State::Export,
        6 => State::Done,
        _ => unreachable!(),
    };
    assert_eq!(state_code, state.to_owned() as u8);
    state
}

/// Pipe runtime context
#[derive(Default)]
pub struct Context {
    state_code: AtomicU8,
    total_run: AtomicU64,
    failure_run: AtomicU64,
}

impl Context {
    pub fn get_state(&self) -> State {
        let code = self.state_code.load(Ordering::Acquire);
        code_to_state(code)
    }

    pub fn get_total_run(&self) -> u64 {
        self.total_run.load(Ordering::Acquire)
    }

    pub fn get_failure_run(&self) -> u64 {
        self.failure_run.load(Ordering::Acquire)
    }

    pub fn set_state(&self, state: State) {
        let code = state as u8;
        self.state_code.store(code, Ordering::Release);
    }

    pub fn inc_total_run(&self) -> u64 {
        self.total_run.fetch_add(1, Ordering::SeqCst)
    }

    pub fn inc_failure_run(&self) -> u64 {
        self.failure_run.fetch_add(1, Ordering::SeqCst)
    }

    pub fn validate(&self, state: State, total_run: u64) {
        assert_eq!(state, self.get_state());
        assert_eq!(total_run, self.get_total_run());
    }
}

#[derive(Deserialize, Serialize)]
pub struct PipeContext {
    name: String,
    state: String,
    total_run: u64,
    failure_run: u64,
}

impl PipeContext {
    pub fn new(name: String, state: State, total_run: u64, failure_run: u64) -> Self {
        PipeContext {
            name: name,
            state: state.to_string(),
            total_run: total_run,
            failure_run: failure_run,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_state(&self) -> &String {
        &self.state
    }

    pub fn get_total_run(&self) -> &u64 {
        &self.total_run
    }
}

impl Display for PipeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ name: {}, state: {}, total_run: {} }}",
            self.name, self.state, self.total_run
        )
    }
}

#[async_trait]
pub trait StoreContext<C>: FromConfig<C> {
    fn store_context(&mut self, pipe_name: String, context: std::sync::Arc<Context>);

    fn load_context(&self, pipe_name: &str) -> Option<&std::sync::Arc<Context>>;

    async fn run(&mut self) -> anyhow::Result<()>;
}

pub struct ContextStore<'a> {
    name: &'a str,
}

impl<'a> ContextStore<'a> {
    pub async fn run<S, C>(
        &mut self,
        config: C,
        contexts: Vec<(String, std::sync::Arc<Context>)>,
    ) -> Result<()>
    where
        S: StoreContext<C>,
        C: ConfigInto<S> + Sync,
    {
        let mut store = config.config_into().await?;
        // add context
        for (name, context) in contexts {
            store.store_context(name, context);
        }
        log::info!("context store {} run ...", self.name);
        store.run().await?;
        log::info!("context store {} exit ...", self.name);
        Ok(())
    }
}

impl<'a> ContextStore<'a> {
    pub fn new(name: &'a str) -> Self {
        ContextStore { name }
    }
}

#[macro_export]
macro_rules! cstore {
    (
        $name:expr
    ) => {{
        ContextStore::new($name)
    }};
}

#[macro_export]
macro_rules! run_cstore {
    (
        $cstore:ident, $config:ty, $path:expr, [$( $pipe:expr ), *]
    ) => {
        {
            let mut contexts = vec![];
            $(
                contexts.push(($pipe.get_name(), $pipe.get_context()));
            )*
            tokio::spawn(async move {
                let config = <$config>::from_path($path).await.expect(&format!("invalid config file location {}", $path));
                match $cstore.run(config, contexts).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!("context store exit with error {:#?}", err);
                        Err(err)
                    }
                }
            })
        }
    };
}

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
    async fn from_config(config: &ContextPrinterConfig) -> anyhow::Result<Self> {
        let delay = match config.delay {
            Some(ref period) => period.to_owned().into(),
            None => Duration::from_micros(0),
        };
        let interval = config.interval.to_owned();
        Ok(ContextPrinter {
            interval: tokio::time::interval(interval.into()),
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
