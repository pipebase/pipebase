use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU64, AtomicU8};

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
    success_run: AtomicU64,
}

impl Context {
    pub fn get_state(&self) -> State {
        let code = self.state_code.load(Ordering::Acquire);
        code_to_state(code)
    }

    pub fn get_total_run(&self) -> u64 {
        self.total_run.load(Ordering::Acquire)
    }

    pub fn get_success_run(&self) -> u64 {
        self.success_run.load(Ordering::Acquire)
    }

    pub fn set_state(&self, state: State) {
        let code = state as u8;
        self.state_code.store(code, Ordering::Release);
    }

    pub fn inc_total_run(&self) -> u64 {
        self.total_run.fetch_add(1, Ordering::SeqCst)
    }

    pub fn inc_success_run(&self) -> u64 {
        self.success_run.fetch_add(1, Ordering::SeqCst)
    }

    pub fn validate(&self, state: State, total_run: u64, success_run: u64) {
        assert_eq!(state, self.get_state());
        assert_eq!(total_run, self.get_total_run());
        assert_eq!(success_run, self.get_success_run());
    }
}

pub trait ContextStore {
    fn add_pipe_context(&mut self, pipe_name: String, context: std::sync::Arc<Context>);
    fn get_pipe_context(&self, pipe_name: &str) -> Option<std::sync::Arc<Context>>;
}
