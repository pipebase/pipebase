use mysql_async::{prelude::*, OptsBuilder, Row};
use pipebase::common::{ConfigInto, FromConfig, FromPath};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MySQLClientConfig {
    url: String,
}

impl FromPath for MySQLClientConfig {}

impl ConfigInto<MySQLClient> for MySQLClientConfig {}

pub struct MySQLClient {
    pool: mysql_async::Pool,
}

impl MySQLClient {
    pub fn new(url: String) -> Self {
        let opts = OptsBuilder::from_opts(url.as_str());
        MySQLClient {
            pool: mysql_async::Pool::new(opts),
        }
    }

    pub async fn execute(&self, statement: &str) -> anyhow::Result<Vec<Row>> {
        let mut conn = self.pool.get_conn().await?;
        let rows: Vec<Row> = conn.exec(statement, mysql_async::Params::Empty).await?;
        Ok(rows)
    }
}

#[async_trait::async_trait]
impl FromConfig<MySQLClientConfig> for MySQLClient {
    async fn from_config(config: MySQLClientConfig) -> anyhow::Result<Self> {
        let url = config.url;
        Ok(MySQLClient::new(url))
    }
}
