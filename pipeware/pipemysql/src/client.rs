use mysql_async::{
    chrono::{Datelike, Duration, Timelike},
    prelude::*,
    OptsBuilder, Params, SslOpts,
};
use pipebase::common::{IntoAttributes, Render, Value};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SslConfig {
    root_cert_path: Option<String>,
}

#[derive(Deserialize)]
pub struct MySQLClientConfig {
    url: String,
    ssl: Option<SslConfig>,
}

pub struct MySQLClient {
    pool: mysql_async::Pool,
}

impl MySQLClient {
    pub fn new(config: MySQLClientConfig) -> Self {
        let url = config.url;
        let ssl = config.ssl;
        let opts = match ssl {
            Some(ssl) => OptsBuilder::from_opts(url.as_str()).ssl_opts(Self::new_ssl_opts(ssl)),
            None => OptsBuilder::from_opts(url.as_str()),
        };
        MySQLClient {
            pool: mysql_async::Pool::new(opts),
        }
    }

    fn new_ssl_opts(ssl: SslConfig) -> SslOpts {
        let root_cert_path = ssl.root_cert_path;
        SslOpts::default().with_root_cert_path(root_cert_path.map(std::path::PathBuf::from))
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
        let params: Vec<Params> = items.into_iter().map(Self::mysql_params).collect();
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
            Value::Integer(value) => match value {
                Some(value) => mysql_async::Value::Int(value.into()),
                None => mysql_async::Value::NULL,
            },
            Value::Long(value) => match value {
                Some(value) => mysql_async::Value::Int(value),
                None => mysql_async::Value::NULL,
            },
            Value::UnsignedInteger(value) => match value {
                Some(value) => mysql_async::Value::UInt(value.into()),
                None => mysql_async::Value::NULL,
            },
            Value::UnsignedLong(value) => match value {
                Some(value) => mysql_async::Value::UInt(value),
                None => mysql_async::Value::NULL,
            },
            Value::Float(value) => match value {
                Some(value) => mysql_async::Value::Float(value),
                None => mysql_async::Value::NULL,
            },
            Value::Double(value) => match value {
                Some(value) => mysql_async::Value::Double(value),
                None => mysql_async::Value::NULL,
            },
            Value::UnsignedBytes(value) => mysql_async::Value::Bytes(value),
            Value::String(value) => match value {
                Some(value) => mysql_async::Value::Bytes(value.into()),
                None => mysql_async::Value::NULL,
            },
            Value::DateTime(value) => match value {
                Some(value) => {
                    let date = value.date();
                    let time = value.time();
                    mysql_async::Value::Date(
                        date.year() as u16,
                        date.month() as u8,
                        date.day() as u8,
                        time.hour() as u8,
                        time.minute() as u8,
                        time.second() as u8,
                        0,
                    )
                }
                None => mysql_async::Value::NULL,
            },
            Value::Duration(value) => match value {
                Some(value) => {
                    let negative = value < Duration::zero();
                    let days = absolute_i64(value.num_days());
                    let hours = absolute_i64(value.num_hours());
                    let minutes = absolute_i64(value.num_minutes());
                    let seconds = absolute_i64(value.num_seconds());
                    let milliseconds = absolute_i64(value.num_milliseconds());
                    let microseconds = absolute_i64(value.num_microseconds().unwrap_or_default());
                    let microseconds = milliseconds * 1000 + microseconds;
                    mysql_async::Value::Time(
                        negative,
                        days as u32,
                        hours as u8,
                        minutes as u8,
                        seconds as u8,
                        microseconds as u32,
                    )
                }
                None => mysql_async::Value::NULL,
            },
            _ => unimplemented!(),
        }
    }
}

// return absolute value of i64
fn absolute_i64(value: i64) -> u64 {
    if value > 0 {
        return value as u64;
    }
    (-value) as u64
}
