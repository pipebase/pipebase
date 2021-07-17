use std::iter::FromIterator;

/// Split T into Collection(V) of U
pub trait Split<T, U, V>
where
    V: FromIterator<U>,
{
    fn split(t: T, pattern: &str) -> V;
}
