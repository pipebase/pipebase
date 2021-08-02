use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{
    atomic::{AtomicU64, AtomicU8, Ordering},
    Arc,
};

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
    #[strum(to_string = "map")]
    Map,
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
        3 => State::Map,
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
            name,
            state: state.to_string(),
            total_run,
            failure_run,
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

pub trait HasContext {
    fn get_name(&self) -> String;
    fn get_context(&self) -> Arc<Context>;
}
