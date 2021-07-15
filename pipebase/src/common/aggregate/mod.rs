mod avg;
mod count;
mod group;
mod init;
mod pair;
mod table;

pub use avg::*;
pub use count::*;
pub use group::*;
pub use init::*;
pub use pair::*;
pub use table::*;

pub trait AggregateAs<T> {
    fn aggregate_value(&self) -> T;
}

pub trait Aggregate<I, T, U>
where
    I: AggregateAs<U>,
    T: IntoIterator<Item = I>,
    U: Init,
{
    fn merge(&self, u: &mut U, i: &I);

    // post merge operation
    fn operate(&self, _u: &mut U) {
        return;
    }

    fn aggregate(&self, t: T) -> U {
        let mut u = U::init();
        for i in t {
            self.merge(&mut u, &i);
        }
        self.operate(&mut u);
        u
    }
}

// u32 as summation result of u32
impl AggregateAs<u32> for u32 {
    fn aggregate_value(&self) -> u32 {
        *self
    }
}

// Vec<u32> as sort result for vec of u32
impl AggregateAs<Vec<u32>> for u32 {
    fn aggregate_value(&self) -> Vec<u32> {
        vec![*self]
    }
}
