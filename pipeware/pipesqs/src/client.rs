use crate::message::*;
use serde::Deserialize;
use sqs::{
    model::{Message, MessageAttributeValue},
    output::ReceiveMessageOutput,
};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SQSClientConfig {
    url: String,
    message_attribute_names: Option<Vec<String>>,
}

pub struct SQSClient {
    client: sqs::Client,
    url: String,
    message_attribute_names: Vec<String>,
}

impl SQSClient {
    pub fn new(config: SQSClientConfig) -> Self {
        SQSClient {
            client: sqs::Client::from_env(),
            url: config.url,
            message_attribute_names: config.message_attribute_names.unwrap_or_default(),
        }
    }

    pub async fn receive_message(&self) -> anyhow::Result<ReceiveMessageOutput> {
        let mut receive_msg = self.client.receive_message().queue_url(&self.url);
        for name in &self.message_attribute_names {
            receive_msg = receive_msg.message_attribute_names(name);
        }
        let msg_output = receive_msg.send().await?;
        Ok(msg_output)
    }

    pub fn handle_message(message: Message) -> SQSMessage {
        let message_attributes = message.message_attributes.unwrap_or_default();
        let body = message.body.unwrap_or_default();
        let message_attribute_values: HashMap<String, SQSMessageAttributeValue> =
            message_attributes
                .into_iter()
                .map(|(name, value)| (name, Self::handle_message_attribute_value(value)))
                .collect();
        SQSMessage {
            body,
            message_attributes: SQSMessageAttributes {
                values: message_attribute_values,
            },
        }
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
