use mysql_async::{prelude::*, Params};
use pipebase::common::{IntoAttributes, Render, Value};
use std::collections::HashMap;

pub struct MySQLClient {
    pool: mysql_async::Pool,
}

impl MySQLClient {
    pub fn new(url: &str) -> Self {
        MySQLClient {
            pool: mysql_async::Pool::new(url),
        }
    }

    pub async fn execute<R>(&self, r: R) -> anyhow::Result<()>
    where
        R: Render,
    {
        let mut conn = self.pool.get_conn().await?;
        let statement = r.render();
        conn.exec_drop(statement, mysql_async::Params::Empty)
            .await?;
        Ok(())
    }

    pub async fn prepare_execute<A>(&self, statement: String, items: Vec<A>) -> anyhow::Result<()>
    where
        A: IntoAttributes,
    {
        let params: Vec<Params> = items
            .into_iter()
            .map(Self::mysql_params)
            .collect();
        let mut conn = self.pool.get_conn().await?;
        conn.exec_batch(statement, params).await?;
        Ok(())
    }

    fn mysql_params<A>(item: A) -> Params
    where
        A: IntoAttributes,
    {
        let attributes: HashMap<String, Value> = item.into_attributes();
        let params: HashMap<String, mysql_async::Value> = attributes
            .into_iter()
            .map(|(name, value)| (name, Self::mysql_value(value)))
            .collect();
        Params::Named(params)
    }

    fn mysql_value(value: Value) -> mysql_async::Value {
        match value {
            Value::Null => mysql_async::Value::NULL,
            Value::Integer(value) => mysql_async::Value::Int(value.into()),
            Value::Long(value) => mysql_async::Value::Int(value),
            Value::UnsignedInteger(value) => mysql_async::Value::UInt(value.into()),
            Value::UnsignedLong(value) => mysql_async::Value::UInt(value),
            Value::Float(value) => mysql_async::Value::Float(value),
            Value::Double(value) => mysql_async::Value::Double(value),
            Value::UnsignedBytes(value) => mysql_async::Value::Bytes(value),
            Value::String(value) => mysql_async::Value::Bytes(value.into()),
            _ => unimplemented!(),
        }
    }
}
