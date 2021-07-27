use pipebase::common::{Averagef32, Convert, Init};
use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedisAveragef32(pub Averagef32);

impl RedisAveragef32 {
    fn new(sum: f32, count: f32) -> Self {
        RedisAveragef32(Averagef32::new(sum, count))
    }

    fn to_literal(&self) -> String {
        let sum = self.0.sum();
        let count = self.0.count();
        format!("{},{}", sum, count)
    }

    fn from_literal(lit: String) -> Self {
        let sum_count: Vec<&str> = lit.split(',').collect();
        let sum: f32 = sum_count
            .get(0)
            .expect("no sum found for RedisAveragef32")
            .parse()
            .expect("failed to parse sum as f32");
        let count: f32 = sum_count
            .get(1)
            .expect("no count found for RedisAveragef32")
            .parse()
            .expect("failed to parse count as f32");
        Self::new(sum, count)
    }
}

impl ToRedisArgs for RedisAveragef32 {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let avg_lit = self.to_literal();
        avg_lit.write_redis_args(out);
    }
}

impl FromRedisValue for RedisAveragef32 {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let lit = String::from_redis_value(v)?;
        let avg = RedisAveragef32::from_literal(lit);
        Ok(avg)
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
