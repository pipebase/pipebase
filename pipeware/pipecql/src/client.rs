use openssl::ssl::{SslContext, SslContextBuilder, SslMethod, SslVerifyMode};
use pipebase::common::{IntoAttributes, Render, Value};
use scylla::{
    frame::value::{SerializedValues, Timestamp},
    prepared_statement::PreparedStatement,
    statement::Consistency,
    transport::session::Session,
    SessionBuilder,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct SslConfig {
    root_cert_path: String,
}

#[derive(Deserialize)]
pub struct CqlClientConfig {
    hostname: String,
    ssl: Option<SslConfig>,
}

pub struct CqlClient {
    session: Session,
}

impl CqlClient {
    pub async fn new(config: CqlClientConfig) -> anyhow::Result<Self> {
        let hostname = config.hostname;
        let ssl = config.ssl;
        let ssl_context = match ssl {
            Some(ssl) => Some(Self::new_ssl_context(ssl)?),
            None => None,
        };
        Ok(CqlClient {
            session: SessionBuilder::new()
                .known_node(hostname)
                .ssl_context(ssl_context)
                .build()
                .await?,
        })
    }

    fn new_ssl_context(ssl: SslConfig) -> anyhow::Result<SslContext> {
        let root_cert_path = ssl.root_cert_path;
        let mut context_builder = SslContextBuilder::new(SslMethod::tls())?;
        let root_cert_path = fs::canonicalize(PathBuf::from(root_cert_path))?;
        context_builder.set_ca_file(root_cert_path.as_path())?;
        context_builder.set_verify(SslVerifyMode::PEER);
        Ok(context_builder.build())
    }

    pub async fn execute<R: Render>(&self, r: R) -> anyhow::Result<()> {
        let prepared = self.session.prepare(r.render()).await?;
        self.session.execute(&prepared, ()).await?;
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
        let mut statement: PreparedStatement = self.session.prepare(statement).await?;
        statement.set_consistency(Consistency::One);
        for item in items {
            let values = Self::serialize_values(item)?;
            self.session.execute(&statement, values).await?;
        }
        Ok(())
    }

    fn serialize_values<A>(item: A) -> anyhow::Result<SerializedValues>
    where
        A: IntoAttributes,
    {
        let attributes = item.into_attribute_tuples();
        let mut values = SerializedValues::with_capacity(attributes.len());
        for (_, value) in attributes {
            Self::serialize_value(&value, &mut values)?;
        }
        Ok(values)
    }

    fn serialize_value(value: &Value, values: &mut SerializedValues) -> anyhow::Result<()> {
        match value {
            Value::Bool(value) => values.add_value(value)?,
            Value::Integer(value) => values.add_value(value)?,
            Value::Long(value) => values.add_value(value)?,
            Value::Float(value) => values.add_value(value)?,
            Value::Double(value) => values.add_value(value)?,
            Value::String(value) => values.add_value(value)?,
            Value::UnsignedBytes(value) => values.add_value(value)?,
            Value::Date(value) => values.add_value(value)?,
            Value::Duration(value) => {
                let ts = value.as_ref().map(|value| Timestamp(*value));
                values.add_value(&ts)?
            }
            _ => unimplemented!(),
        };
        Ok(())
    }
}
