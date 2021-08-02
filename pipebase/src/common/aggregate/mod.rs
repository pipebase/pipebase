mod avg;
mod count;
mod group;
mod init;
mod pair;

pub use avg::*;
pub use count::*;
pub use group::*;
pub use init::*;
pub use pair::*;

pub trait AggregateAs<T> {
    /// Get value to aggregate
    fn aggregate_value(&self) -> T;
}

/// Aggregate items
pub trait Aggregate<I, T, U>
where
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
    U: Init,
{
    /// Merge items
    fn merge(&self, u: &mut U, i: &I);

    /// Post merge operation
    fn operate(&self, _u: &mut U) {}

    /// Aggregate items
    /// * Merge
    /// * Post merge operation
    fn aggregate(&self, t: T) -> U {
        let mut u = U::init();
        for i in t {
            self.merge(&mut u, &i);
        }
        self.operate(&mut u);
        u
    }
}

// u32 aggregate value for summation
impl AggregateAs<u32> for u32 {
    fn aggregate_value(&self) -> u32 {
        *self
    }
}

// u32 aggregate value for sorting
impl AggregateAs<Vec<u32>> for u32 {
    fn aggregate_value(&self) -> Vec<u32> {
        vec![*self]
    }
}
