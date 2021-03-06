use dynamodb::model::AttributeValue;
use dynamodb::{Blob, Client, Config, Region};
use pipebase::common::{IntoAttributes, Value};
use std::collections::HashMap;

pub struct DynamocDBClient {
    client: Client,
    table: String,
}

impl DynamocDBClient {
    pub fn new(region: String, table: String) -> Self {
        let region = Region::new(region);
        let config = Config::builder().region(region).build();
        let client = Client::from_conf(config);
        DynamocDBClient { client, table }
    }

    pub async fn put_item<T>(&self, attribute_values: T) -> anyhow::Result<()>
    where
        T: IntoAttributes,
    {
        let attribute_values: HashMap<String, AttributeValue> = attribute_values
            .into_attributes()
            .into_iter()
            .map(|(k, v)| (k, Self::convert_attribute_value(v)))
            .collect();
        let mut request = self.client.put_item().table_name(&self.table);
        for (name, value) in attribute_values {
            request = request.item(name, value);
        }
        request.send().await?;
        Ok(())
    }

    pub fn convert_attribute_value(v: Value) -> AttributeValue {
        match v {
            Value::Bool(v) => match v {
                Some(v) => AttributeValue::Bool(v),
                None => AttributeValue::Null(true),
            },
            Value::UnsignedInteger(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::Integer(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::UnsignedLong(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::Long(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::Float(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::Double(v) => match v {
                Some(v) => AttributeValue::N(format!("{}", v)),
                None => AttributeValue::Null(true),
            },
            Value::String(v) => match v {
                Some(v) => AttributeValue::S(v),
                None => AttributeValue::Null(true),
            },
            Value::UnsignedBytes(bs) => AttributeValue::B(Blob::new(bs)),
            Value::Array(vs) => {
                AttributeValue::L(vs.into_iter().map(Self::convert_attribute_value).collect())
            }
            Value::Attributes(vs) => AttributeValue::M(
                vs.into_iter()
                    .map(|(k, v)| (k, Self::convert_attribute_value(v)))
                    .collect(),
            ),
            _ => unimplemented!(),
        }
    }
}
