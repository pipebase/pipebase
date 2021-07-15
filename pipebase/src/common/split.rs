use std::iter::FromIterator;
pub trait Split<T, U, V>
where
    V: FromIterator<U>,
{
    fn split(t: T, pattern: &str) -> V;
}
