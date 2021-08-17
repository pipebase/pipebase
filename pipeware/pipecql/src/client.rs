use pipebase::common::{IntoAttributes, Render, Value};
use scylla::{
    frame::value::SerializedValues, prepared_statement::PreparedStatement, statement::Consistency,
    transport::session::Session, SessionBuilder,
};

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
            _ => unimplemented!(),
        };
        Ok(())
    }
}
