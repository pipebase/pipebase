use pipebase::common::{Convert, Count32, Init};
use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedisCount32(Count32);

impl RedisCount32 {
    fn new(c: u32) -> Self {
        RedisCount32(Count32::new(c))
    }
}

impl ToRedisArgs for RedisCount32 {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        self.0.get().write_redis_args(out)
    }
}

impl FromRedisValue for RedisCount32 {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let c = u32::from_redis_value(v)?;
        Ok(Self::new(c))
    }
}

impl Init for RedisCount32 {
    fn init() -> Self {
        RedisCount32(Count32::init())
    }
}

impl From<Count32> for RedisCount32 {
    fn from(count32: Count32) -> Self {
        RedisCount32(count32)
    }
}

impl Convert<RedisCount32> for Count32 {
    fn convert(rhs: RedisCount32) -> Self {
        rhs.0
    }
}

impl AddAssign for RedisCount32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}
