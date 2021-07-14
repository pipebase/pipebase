use pipebase::Render;
use scylla::transport::session::Session;
use scylla::SessionBuilder;

pub struct CqlClient {
    session: Session,
}

impl CqlClient {
    pub async fn new<H: AsRef<str>>(hostname: H) -> anyhow::Result<Self> {
        Ok(CqlClient {
            session: SessionBuilder::new().known_node(hostname).build().await?,
        })
    }

    pub async fn execute<R: Render>(&self, r: R) -> anyhow::Result<()> {
        let prepared = self.session.prepare(r.render()).await?;
        self.session.execute(&prepared, ()).await?;
        Ok(())
    }
}
