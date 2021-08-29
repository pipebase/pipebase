use crate::client::{new_client, qos, ClientOptions, QoSType};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use rumqttc::{AsyncClient, QoS};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MqttPublisherConfig {
    base: ClientOptions,
    topic: String,
    qos: QoSType,
    retain: bool,
}

impl FromPath for MqttPublisherConfig {}

impl ConfigInto<MqttPublisher> for MqttPublisherConfig {}

pub struct MqttPublisher {
    client: AsyncClient,
    topic: String,
    qos: QoS,
    retain: bool,
}

#[async_trait]
impl FromConfig<MqttPublisherConfig> for MqttPublisher {
    async fn from_config(config: MqttPublisherConfig) -> anyhow::Result<Self> {
        let qos = qos(config.qos);
        let topic = config.topic;
        let retain = config.retain;
        let (client, mut event) = new_client(&config.base);
        // keep channel open
        tokio::spawn(async move {
            loop {
                let notification = event.poll().await.unwrap();
                log::info!("MqttPublisher received = {:?}", notification);
            }
        });
        Ok(MqttPublisher {
            client,
            topic,
            qos,
            retain,
        })
    }
}

#[async_trait]
impl Export<Vec<u8>, MqttPublisherConfig> for MqttPublisher {
    async fn export(&mut self, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .publish(&self.topic, self.qos, self.retain, payload)
            .await?;
        Ok(())
    }
}
