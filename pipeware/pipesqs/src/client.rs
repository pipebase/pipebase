use crate::message::*;
use serde::Deserialize;
use sqs::{
    config::Config,
    model::{Message, MessageAttributeValue},
    output::{DeleteMessageOutput, ReceiveMessageOutput},
};
use std::collections::HashMap;
use tracing::error;

#[derive(Deserialize)]
pub struct SqsClientConfig {
    url: String,
    message_attribute_names: Option<Vec<String>>,
}

pub struct SqsClient {
    client: sqs::Client,
    url: String,
    message_attribute_names: Vec<String>,
}

impl SqsClient {
    pub fn new(config: SqsClientConfig) -> Self {
        let conf = Config::builder().build();
        SqsClient {
            client: sqs::Client::from_conf(conf),
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

    pub async fn handle_message(&self, message: Message) -> SqsMessage {
        let message_attributes = message.message_attributes.unwrap_or_default();
        let body = message.body.unwrap_or_default();
        let message_attribute_values: HashMap<String, SqsMessageAttributeValue> =
            message_attributes
                .into_iter()
                .map(|(name, value)| (name, Self::handle_message_attribute_value(value)))
                .collect();
        // Amazon SQS doesn't automatically delete the message
        // consumer must delete the message from the queue after receiving and processing it
        if let Some(receipt_handle) = message.receipt_handle {
            if let Err(e) = self.delete_message(receipt_handle).await {
                error!("delete message error '{}'", e)
            }
        }
        SqsMessage {
            body,
            message_attributes: SqsMessageAttributes {
                values: message_attribute_values,
            },
        }
    }

    fn handle_message_attribute_value(value: MessageAttributeValue) -> SqsMessageAttributeValue {
        let ty = value.data_type.unwrap_or_default();
        if let Some(string_value) = value.string_value {
            return SqsMessageAttributeValue {
                ty,
                data: SqsMessageAttributeData::String(string_value),
            };
        }
        if let Some(blob) = value.binary_value {
            return SqsMessageAttributeValue {
                ty,
                data: SqsMessageAttributeData::Binary(blob.into_inner()),
            };
        }
        unimplemented!("handle MessageAttributeValue not implemented")
    }
}
