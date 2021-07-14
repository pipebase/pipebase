use crate::utils::PsqlClient;
use async_trait::async_trait;
use pipebase::{ConfigInto, Export, FromConfig, FromPath, Render};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PsqlWriterConfig {
    params: String,
}

impl FromPath for PsqlWriterConfig {}

impl ConfigInto<PsqlWriter> for PsqlWriterConfig {}

pub struct PsqlWriter {
    client: PsqlClient,
}

#[async_trait]
impl FromConfig<PsqlWriterConfig> for PsqlWriter {
    async fn from_config(config: &PsqlWriterConfig) -> anyhow::Result<Self> {
        Ok(PsqlWriter {
            // TODO: Support Tls
            client: PsqlClient::new(&config.params).await?,
        })
    }
}

#[async_trait]
impl<T> Export<T, PsqlWriterConfig> for PsqlWriter
where
    T: Render + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.execute(t).await
    }
}

#[cfg(test)]
mod psql_tests {
    use pipebase::*;

    #[derive(Debug, Clone, Render)]
    #[render(
        template = r#"INSERT INTO records (key, value) VALUES ('{}', {}) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"#
    )]
    struct Record {
        #[render(pos = 2)]
        value: i32,
        #[render(pos = 1)]
        key: String,
    }

    #[tokio::test]
    #[ignore = "move to itest"]
    async fn test_psql() {
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
            value: 1,
        };
        let statement = record.render();
        let _ = client
            .execute(&statement[..], &[])
            .await
            .expect("statement failed");
        let record = Record {
            key: "foo".to_owned(),
            value: 2,
        };
        let statement = record.render();
        let _ = client
            .execute(&statement[..], &[])
            .await
            .expect("statement failed");
    }
}
