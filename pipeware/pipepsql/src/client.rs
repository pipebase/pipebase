use pipebase::common::{IntoAttributes, Render, Value};
use tokio_postgres::{types::ToSql, Client, NoTls};
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
