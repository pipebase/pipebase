use pipebase::common::Render;
use tokio_postgres::{Client, NoTls};
pub struct PsqlClient {
    client: Client,
}

impl PsqlClient {
    // params schema: https://github.com/sfackler/rust-postgres/blob/master/postgres/src/config.rs
    // type supoort: https://docs.rs/postgres/0.19.1/postgres/types/trait.ToSql.html
    pub async fn new(params: String) -> anyhow::Result<Self> {
        let (client, connection) = tokio_postgres::connect(&params, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(PsqlClient { client })
    }

    pub async fn execute<R>(&mut self, r: R) -> anyhow::Result<()>
    where
        R: Render,
    {
        let statement = r.render();
        let rows_updated = self.client.execute(&statement[..], &[]).await?;
        log::info!("{} rows updated", rows_updated);
        Ok(())
    }
}
