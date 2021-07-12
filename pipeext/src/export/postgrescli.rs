use async_trait::async_trait;
use pipebase::{ConfigInto, Export, FromConfig, FromPath};
use serde::Deserialize;

pub trait Psql {
    fn psql(&self) -> String;
}

#[derive(Deserialize)]
pub struct PsqlWriterConfig {
    params: String,
}

impl FromPath for PsqlWriterConfig {}

impl ConfigInto<PsqlWriter> for PsqlWriterConfig {}

pub struct PsqlWriter {
    client: tokio_postgres::Client,
}

#[async_trait]
impl FromConfig<PsqlWriterConfig> for PsqlWriter {
    async fn from_config(config: &PsqlWriterConfig) -> anyhow::Result<Self> {
        let (client, connection) =
            tokio_postgres::connect(&config.params, tokio_postgres::NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(PsqlWriter {
            // TODO: Support Tls
            client: client,
        })
    }
}

#[async_trait]
impl<T> Export<T, PsqlWriterConfig> for PsqlWriter
where
    T: Psql + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.execute(t).await
    }
}

impl PsqlWriter {
    async fn execute<R>(&mut self, record: R) -> anyhow::Result<()>
    where
        R: Psql,
    {
        let statement = record.psql();
        let rows_updated = self.client.execute(&statement[..], &[]).await?;
        log::info!("{} rows updated", rows_updated);
        Ok(())
    }
}

#[cfg(test)]
mod psql_tests {
    use crate::*;
    use pipebase::*;

    #[derive(Debug, Clone, Psql)]
    #[psql(
        query = r#"INSERT INTO records (key, value) VALUES ('{}', {}) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"#
    )]
    struct Record {
        #[psql(pos = 2)]
        value: i32,
        #[psql(pos = 1)]
        key: String,
    }

    #[tokio::test]
    #[ignore = "move to itest"]
    async fn test_psql() {
        // config https://github.com/sfackler/rust-postgres/blob/master/postgres/src/config.rs
        // type https://docs.rs/postgres/0.19.1/postgres/types/trait.ToSql.html

        let (client, connection) = tokio_postgres::connect(
            "host=localhost port=5432 user=postgres password=postgres dbname=postgres",
            tokio_postgres::NoTls,
        )
        .await
        .expect("connection failure");
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        client
            .batch_execute(
                "
            CREATE TABLE IF NOT EXISTS records (
                key TEXT PRIMARY KEY,
                value   INTEGER
            )
        ",
            )
            .await
            .expect("falied to create table");

        let record = Record {
            key: "foo".to_owned(),
            value: 5,
        };
        let statement = record.psql();
        let _ = client
            .execute(&statement[..], &[])
            .await
            .expect("statement failed");
        let record = Record {
            key: "foo".to_owned(),
            value: 6,
        };
        let statement = record.psql();
        let _ = client
            .execute(&statement[..], &[])
            .await
            .expect("statement failed");
    }
}
