use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

use super::AggregateAs;

// Average is (sum, count) pair
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Averagef32(pub f32, pub f32);

impl Averagef32 {
    pub fn new(sum: f32, count: f32) -> Self {
        Averagef32(sum, count)
    }

    pub fn average(&self) -> f32 {
        assert_ne!(self.1, 0.0, "divide by zero");
        self.0 / self.1
    }

    pub fn sum(&self) -> f32 {
        self.0
    }

    pub fn count(&self) -> f32 {
        self.1
    }
}

impl AggregateAs<Averagef32> for u32 {
    fn aggregate_value(&self) -> Averagef32 {
        Averagef32(*self as f32, 1.0)
    }
}

impl AddAssign<Averagef32> for Averagef32 {
    fn add_assign(&mut self, rhs: Averagef32) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}
