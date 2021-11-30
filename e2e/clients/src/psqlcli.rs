use pipebase::common::{ConfigInto, FromConfig, FromPath};
use serde::Deserialize;
use tokio_postgres::{Client, NoTls, Row};

#[derive(Deserialize)]
pub struct PsqlClientConfig {
    params: String,
}

impl FromPath for PsqlClientConfig {}

impl ConfigInto<PsqlClient> for PsqlClientConfig {}

pub struct PsqlClient {
    client: Client,
}

impl PsqlClient {
    // params schema: https://github.com/sfackler/rust-postgres/blob/master/postgres/src/config.rs
    // type supoort: https://docs.rs/postgres/0.19.1/postgres/types/trait.ToSql.html
    pub async fn new(params: String) -> anyhow::Result<Self> {
        let client = Self::connect(params).await?;
        Ok(PsqlClient { client })
    }

    async fn connect(params: String) -> anyhow::Result<Client> {
        let (client, connection) = tokio_postgres::connect(&params, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(client)
    }

    pub async fn query(&self, query: &str) -> anyhow::Result<Vec<Row>> {
        let rows = self.client.query(query, &[]).await?;
        Ok(rows)
    }

    pub async fn execute(&self, statement: &str) -> anyhow::Result<()> {
        let _ = self.client.execute(statement, &[]).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl FromConfig<PsqlClientConfig> for PsqlClient {
    async fn from_config(config: PsqlClientConfig) -> anyhow::Result<Self> {
        let params = config.params;
        PsqlClient::new(params).await
    }
}
