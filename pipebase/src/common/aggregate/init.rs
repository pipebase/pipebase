use super::{Averagef32, Count32};

pub trait Init {
    fn init() -> Self;
}

impl<T> Init for Vec<T> {
    fn init() -> Self {
        Vec::new()
    }
}

impl Init for u32 {
    fn init() -> u32 {
        0
    }
}

impl Init for Count32 {
    fn init() -> Count32 {
        Count32::new(0)
    }
}

impl Init for Averagef32 {
    fn init() -> Self {
        Averagef32::new(0.0, 0.0)
    }
}
