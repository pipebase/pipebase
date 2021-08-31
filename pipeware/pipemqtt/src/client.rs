use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub enum QoSType {
    AtMost,
    AtLeast,
    Exactly,
}

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Clone, Deserialize)]
pub struct LastWill {
    topic: String,
    message: String,
    qos: QoSType,
    retain: bool,
}

#[derive(Deserialize)]
pub struct ClientOptions {
    id: String,
    host: String,
    port: u16,
    cap: usize,
    // number of seconds after which client should ping the broker if there is no other data exchange
    keep_alive: u16,
    credentials: Option<Credentials>,
    last_will: Option<LastWill>,
}

pub(crate) fn new_client(options: &ClientOptions) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(&options.id, &options.host, options.port);
    mqttoptions.set_keep_alive(options.keep_alive);
    if let Some(ref credentials) = options.credentials {
        let username = &credentials.username;
        let password = &credentials.password;
        mqttoptions.set_credentials(username, password);
    }
    if let Some(ref last_will) = options.last_will {
        let last_will = to_last_will(last_will.to_owned());
        mqttoptions.set_last_will(last_will);
    }
    AsyncClient::new(mqttoptions, options.cap)
}

pub(crate) fn to_qos(qos: QoSType) -> QoS {
    match qos {
        QoSType::AtLeast => QoS::AtLeastOnce,
        QoSType::AtMost => QoS::AtMostOnce,
        QoSType::Exactly => QoS::ExactlyOnce,
    }
}

pub(crate) fn to_last_will(last_will: LastWill) -> rumqttc::LastWill {
    rumqttc::LastWill {
        topic: last_will.topic,
        message: last_will.message.into(),
        qos: to_qos(last_will.qos),
        retain: last_will.retain,
    }
}
