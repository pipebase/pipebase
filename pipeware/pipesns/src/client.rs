use serde::Deserialize;
use sns::{Client, Config, Region};

#[derive(Deserialize)]
pub struct Subscriber {
    protocol: String,
    endpoint: String,
}

pub struct SnsClient {
    client: Client,
    topic_arn: String,
}

impl SnsClient {
    pub async fn new(
        region: String,
        topic_arn: String,
        subscribers: Vec<Subscriber>,
    ) -> anyhow::Result<Self> {
        let region = Region::new(region);
        let conf = Config::builder().region(region).build();
        let client = Client::from_conf(conf);
        for subscriber in subscribers {
            let protocol = subscriber.protocol;
            let endpoint = subscriber.endpoint;
            client
                .subscribe()
                .topic_arn(&topic_arn)
                .protocol(protocol)
                .endpoint(endpoint)
                .send()
                .await?;
        }
        Ok(SnsClient { client, topic_arn })
    }

    pub async fn publish<T>(&self, t: T) -> anyhow::Result<()>
    where
        T: Into<String>,
    {
        self.client
            .publish()
            .topic_arn(&self.topic_arn)
            .message(t)
            .send()
            .await?;
        Ok(())
    }
}
