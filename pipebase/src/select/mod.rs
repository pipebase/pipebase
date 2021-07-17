mod hash;
mod random;
mod roundrobin;
mod runtime;

pub use hash::*;
pub use random::*;
pub use roundrobin::*;
pub use runtime::*;

use crate::FromConfig;

pub trait Select<T, C>: Send + Sync + FromConfig<C> {
    fn select(&mut self, t: &T, candidates: &[&usize]) -> Vec<usize>;
}
