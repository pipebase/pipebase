use pipebase::common::{ConfigInto, FromConfig, FromPath};
use scylla::{transport::session::Session, QueryResult, SessionBuilder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CqlClientConfig {
    hostname: String,
}

impl FromPath for CqlClientConfig {}

pub struct CqlClient {
    session: Session,
}

impl ConfigInto<CqlClient> for CqlClientConfig {}

impl CqlClient {
    pub async fn new(hostname: String) -> anyhow::Result<Self> {
        Ok(CqlClient {
            session: SessionBuilder::new().known_node(hostname).build().await?,
        })
    }

    pub async fn execute(&self, statement: &str) -> anyhow::Result<QueryResult> {
        let prepared = self.session.prepare(statement).await?;
        let result = self.session.execute(&prepared, ()).await?;
        Ok(result)
    }
}

#[async_trait::async_trait]
impl FromConfig<CqlClientConfig> for CqlClient {
    async fn from_config(config: CqlClientConfig) -> anyhow::Result<Self> {
        CqlClient::new(config.hostname).await
    }
}
