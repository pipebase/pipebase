use crate::{AggregateAs, Init};
use std::{cmp::Ordering, fmt::Debug};

#[derive(Clone, Debug, Eq)]
pub struct Count32(u32);

impl Count32 {
    pub fn new(c: u32) -> Self {
        Count32(c)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl std::ops::AddAssign<Self> for Count32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Ord for Count32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Count32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl PartialEq for Count32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Init for Count32 {
    fn init() -> Count32 {
        Count32(0)
    }
}

impl AggregateAs<Count32> for Count32 {
    fn aggregate_value(&self) -> Count32 {
        self.to_owned()
    }
}

impl AggregateAs<Vec<Count32>> for Count32 {
    fn aggregate_value(&self) -> Vec<Count32> {
        vec![self.to_owned()]
    }
}
