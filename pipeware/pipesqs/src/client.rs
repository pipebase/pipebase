use serde::Deserialize;
use sqs::{
    model::{Message, MessageAttributeValue},
    output::ReceiveMessageOutput,
};
use std::collections::HashMap;

pub enum SQSMessageAttributeData {
    String(String),
    Binary(Vec<u8>),
}

pub struct SQSMessageAttributeValue {
    pub ty: String,
    pub data: SQSMessageAttributeData,
}

#[derive(Deserialize)]
pub struct SQSClientConfig {
    url: String,
}

pub struct SQSClient {
    client: sqs::Client,
    url: String,
}

impl SQSClient {
    pub fn new(config: SQSClientConfig) -> Self {
        SQSClient {
            client: sqs::Client::from_env(),
            url: config.url,
        }
    }

    pub async fn receive_message(&self) -> anyhow::Result<ReceiveMessageOutput> {
        let msg_output = self
            .client
            .receive_message()
            .queue_url(&self.url)
            .send()
            .await?;
        Ok(msg_output)
    }

    pub fn handle_message(message: Message) -> (String, HashMap<String, SQSMessageAttributeValue>) {
        let message_attributes = message.message_attributes.unwrap_or_default();
        let body = message.body.unwrap_or_default();
        let message_attributes: HashMap<String, SQSMessageAttributeValue> = message_attributes
            .into_iter()
            .map(|(name, value)| (name, Self::handle_message_attribute_value(value)))
            .collect();
        (body, message_attributes)
    }

    fn handle_message_attribute_value(value: MessageAttributeValue) -> SQSMessageAttributeValue {
        let ty = value.data_type.unwrap_or_default();
        match value.string_value {
            Some(string_value) => {
                return SQSMessageAttributeValue {
                    ty,
                    data: SQSMessageAttributeData::String(string_value),
                }
            }
            None => (),
        };
        match value.binary_value {
            Some(blob) => {
                return SQSMessageAttributeValue {
                    ty,
                    data: SQSMessageAttributeData::Binary(blob.into_inner()),
                }
            }
            None => (),
        };
        unimplemented!("handle MessageAttributeValue not implemented")
    }
}
