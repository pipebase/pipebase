use mysql_async::prelude::*;
use pipebase::common::Render;

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
}
