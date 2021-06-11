use strum::{Display, EnumString};

#[derive(Clone, Display, EnumString)]

/// Pipe running state
#[derive(PartialEq, Debug)]
pub enum State {
    #[strum(to_string = "init")]
    Init,
    #[strum(to_string = "receive")]
    Receive,
    #[strum(to_string = "poll")]
    Poll,
    #[strum(to_string = "process")]
    Process,
    #[strum(to_string = "send")]
    Send,
    #[strum(to_string = "done")]
    Done,
}

impl Default for State {
    fn default() -> Self {
        State::Init
    }
}

/// Pipe runtime context
#[derive(Clone, Default)]
pub struct Context {
    state: State,
    total_run: u128,
    success_run: u128,
}

impl Context {
    pub fn get_state(&self) -> State {
        self.state.to_owned()
    }

    pub fn get_total_run(&self) -> u128 {
        self.total_run
    }

    pub fn get_success_run(&self) -> u128 {
        self.success_run
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    pub fn inc_total_run(&mut self) {
        self.total_run += 1
    }

    pub fn inc_success_run(&mut self) {
        self.success_run += 1
    }

    pub fn validate(&self, state: State, total_run: u128, success_run: u128) {
        assert_eq!(state, self.state);
        assert_eq!(total_run, self.total_run);
        assert_eq!(success_run, self.success_run);
    }
}
