use std::iter::FromIterator;

use super::Map;
use crate::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;

pub trait Split<T, U, V> 
where
    V: FromIterator<U>
{
    fn split(t: T) -> V; 
}