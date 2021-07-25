use crate::message::*;
use serde::Deserialize;
use sqs::{
    model::{Message, MessageAttributeValue},
    output::{DeleteMessageOutput, ReceiveMessageOutput},
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
        let receive_msg_output = receive_msg.send().await?;
        Ok(receive_msg_output)
    }

    pub async fn delete_message(
        &self,
        receipt_handle: String,
    ) -> anyhow::Result<DeleteMessageOutput> {
        let delete_msg = self
            .client
            .delete_message()
            .queue_url(&self.url)
            .receipt_handle(receipt_handle);
        let delete_msg_output = delete_msg.send().await?;
        Ok(delete_msg_output)
    }

    pub async fn handle_message(&self, message: Message) -> SQSMessage {
        let message_attributes = message.message_attributes.unwrap_or_default();
        let body = message.body.unwrap_or_default();
        let message_attribute_values: HashMap<String, SQSMessageAttributeValue> =
            message_attributes
                .into_iter()
                .map(|(name, value)| (name, Self::handle_message_attribute_value(value)))
                .collect();
        // Amazon SQS doesn't automatically delete the message
        // consumer must delete the message from the queue after receiving and processing it
        match message.receipt_handle {
            Some(receipt_handle) => {
                match self.delete_message(receipt_handle).await {
                    Ok(_) => (),
                    Err(e) => log::error!("delete message error '{}'", e),
                };
            }
            None => (),
        };
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
