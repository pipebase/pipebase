use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum QoSType {
    AtMost,
    AtLeast,
    Exactly,
}

#[derive(Deserialize)]
pub struct ClientOptions {
    id: String,
    host: String,
    port: u16,
    cap: usize,
    // number of seconds after which client should ping the broker if there is no other data exchange
    keep_alive: u16,
}

pub(crate) fn new_client(options: &ClientOptions) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(&options.id, &options.host, options.port);
    mqttoptions.set_keep_alive(options.keep_alive);
    AsyncClient::new(mqttoptions, options.cap)
}

pub(crate) fn qos(qos: QoSType) -> QoS {
    match qos {
        QoSType::AtLeast => QoS::AtLeastOnce,
        QoSType::AtMost => QoS::AtMostOnce,
        QoSType::Exactly => QoS::ExactlyOnce,
    }
}
