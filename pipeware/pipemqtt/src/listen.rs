use crate::client::{new_client, to_qos, ClientOptions, QoSType};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use rumqttc::{Event, EventLoop, Packet, QoS};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use tracing::info;

#[derive(Deserialize)]
pub struct MqttSubscriberConfig {
    base: ClientOptions,
    topic: String,
    qos: QoSType,
}

impl FromPath for MqttSubscriberConfig {}

impl ConfigInto<MqttSubscriber> for MqttSubscriberConfig {}

pub struct MqttSubscriber {
    topic: String,
    qos: QoS,
    client_opts: ClientOptions,
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FromConfig<MqttSubscriberConfig> for MqttSubscriber {
    async fn from_config(config: MqttSubscriberConfig) -> anyhow::Result<Self> {
        let client_opts = config.base;
        let topic = config.topic;
        let qos = to_qos(config.qos);
        Ok(MqttSubscriber {
            topic,
            qos,
            client_opts,
            tx: None,
        })
    }
}

#[async_trait]
impl Listen<Vec<u8>, MqttSubscriberConfig> for MqttSubscriber {
    async fn run(&mut self) -> anyhow::Result<()> {
        let (client, event) = new_client(&self.client_opts);
        client.subscribe(&self.topic, self.qos).await?;
        let tx = self
            .tx
            .as_ref()
            .expect("sender not inited for mqtt listener");
        Self::start_loop(event, tx).await
    }

    fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}

impl MqttSubscriber {
    async fn start_loop(mut event: EventLoop, tx: &Sender<Vec<u8>>) -> anyhow::Result<()> {
        loop {
            let event = event.poll().await?;
            let packet = match event {
                Event::Incoming(packet) => packet,
                _ => continue,
            };
            let payload = match packet {
                Packet::Publish(publish) => publish.payload,
                _ => {
                    info!("incoming packet {:?}", packet);
                    continue;
                }
            };
            tx.send(payload.to_vec()).await?;
        }
    }
}
