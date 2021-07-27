use pipebase::common::{Averagef32, Convert, Init};
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
        let sum = self.0.sum();
        let count = self.0.count();
        let avg: Vec<f32> = vec![sum, count];
        avg.write_redis_args(out);
    }

    fn is_single_arg(&self) -> bool {
        false
    }
}

impl FromRedisValue for RedisAveragef32 {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let sum_and_count = Vec::<f32>::from_redis_value(v)?;
        let sum = sum_and_count.get(0).expect("sum not found");
        let count = sum_and_count.get(1).expect("count not found");
        let avgf32 = Averagef32::new(sum.to_owned(), count.to_owned());
        Ok(RedisAveragef32(avgf32))
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
