use openssl::ssl::{SslConnector, SslMethod};
use pipebase::common::{IntoAttributes, Render, Value};
use postgres_openssl::MakeTlsConnector;
use serde::Deserialize;
use tokio_postgres::{types::ToSql, Client, NoTls};
use tracing::info;

#[derive(Deserialize)]
pub struct SslConfig {
    root_cert_path: String,
}

#[derive(Deserialize)]
pub struct PsqlClientConfig {
    params: String,
    ssl: Option<SslConfig>,
}

pub struct PsqlClient {
    client: Client,
}

impl PsqlClient {
    // params schema: https://github.com/sfackler/rust-postgres/blob/master/postgres/src/config.rs
    // type supoort: https://docs.rs/postgres/0.19.1/postgres/types/trait.ToSql.html
    pub async fn new(config: PsqlClientConfig) -> anyhow::Result<Self> {
        let params = config.params;
        let ssl = config.ssl;
        let client = match ssl {
            Some(ssl) => Self::connect_tls(params, ssl).await?,
            None => Self::connect(params).await?,
        };
        Ok(PsqlClient { client })
    }

    fn make_tls(ssl: SslConfig) -> anyhow::Result<MakeTlsConnector> {
        let root_cert_path = ssl.root_cert_path;
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_ca_file(&root_cert_path)?;
        let connector = MakeTlsConnector::new(builder.build());
        Ok(connector)
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

    async fn connect_tls(params: String, ssl: SslConfig) -> anyhow::Result<Client> {
        let connector = Self::make_tls(ssl)?;
        let (client, connection) = tokio_postgres::connect(&params, connector).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(client)
    }

    pub async fn execute<R>(&mut self, r: R) -> anyhow::Result<()>
    where
        R: Render,
    {
        let statement = r.render();
        let rows_updated = self.client.execute(&statement[..], &[]).await?;
        info!("{} rows updated", rows_updated);
        Ok(())
    }

    pub async fn prepare_execute<A>(
        &mut self,
        statement: String,
        items: Vec<A>,
    ) -> anyhow::Result<()>
    where
        A: IntoAttributes,
    {
        let prepared_statement = self.client.prepare(&statement).await?;
        for item in items {
            let params = Self::from_attributes_to_params(item);
            let params: Vec<&(dyn ToSql + Send + Sync)> =
                params.iter().map(|p| p.as_ref()).collect();
            self.client
                .execute_raw(&prepared_statement, slice_iter(&params))
                .await?;
        }
        Ok(())
    }

    fn from_attributes_to_params<A>(attributes: A) -> Vec<Box<dyn ToSql + Send + Sync>>
    where
        A: IntoAttributes,
    {
        attributes
            .into_attribute_tuples()
            .into_iter()
            .map(|(_, value)| Self::psql_value(value))
            .collect()
    }

    fn psql_value(value: Value) -> Box<dyn ToSql + Send + Sync> {
        match value {
            Value::Bool(value) => Box::new(value),
            Value::UnsignedInteger(value) => Box::new(value),
            Value::Integer(value) => Box::new(value),
            Value::Long(value) => Box::new(value),
            Value::Float(value) => Box::new(value),
            Value::Double(value) => Box::new(value),
            Value::String(value) => Box::new(value),
            Value::UnsignedBytes(value) => Box::new(value),
            Value::Date(value) => Box::new(value),
            Value::DateTime(value) => Box::new(value),
            Value::UtcTime(value) => Box::new(value),
            Value::LocalTime(value) => Box::new(value),
            _ => unimplemented!(),
        }
    }
}

fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Send + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
