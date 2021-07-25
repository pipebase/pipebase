use serde::Deserialize;
use sqs::output::ReceiveMessageOutput;

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
}
