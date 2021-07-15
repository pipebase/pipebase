pub trait Filter<Rhs = Self> {
    fn filter(rhs: &Rhs) -> bool;
}
