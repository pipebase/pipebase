use pipebase::common::{Averagef32, Convert, Init};
use pipebytes::{FromBytes, IntoBytes};
use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedisAveragef32(pub Averagef32);

impl ToRedisArgs for RedisAveragef32 {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = self
            .0
            .into_bytes()
            .expect("failed to encode Averagef32 as bytes");
        out.write_arg(&bytes);
    }
}

impl FromRedisValue for RedisAveragef32 {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let bytes = Vec::<u8>::from_redis_value(v)?;
        let avg = Averagef32::from_bytes(bytes).expect("failed to decode Averagef32 from bytes");
        Ok(RedisAveragef32(avg))
    }
}

impl From<Averagef32> for RedisAveragef32 {
    fn from(avg: Averagef32) -> Self {
        RedisAveragef32(avg)
    }
}

impl Convert<RedisAveragef32> for Averagef32 {
    fn convert(rhs: RedisAveragef32) -> Self {
        rhs.0
    }
}

impl AddAssign<RedisAveragef32> for RedisAveragef32 {
    fn add_assign(&mut self, rhs: RedisAveragef32) {
        self.0 += rhs.0;
    }
}

impl Init for RedisAveragef32 {
    fn init() -> Self {
        RedisAveragef32(Averagef32::init())
    }
}
