use async_trait::async_trait;
use log::error;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::Interval;

use crate::ConfigInto;
use crate::FromConfig;
use crate::FromFile;
use crate::{spawn_send, wait_join_handles};
use crate::{Listen, Poll};

#[derive(Deserialize)]
pub struct TimePollerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}

impl FromFile for TimePollerConfig {}

#[derive(Deserialize)]
pub struct TimeListenerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}

impl FromFile for TimeListenerConfig {}

#[async_trait]
impl ConfigInto<TimeListener> for TimeListenerConfig {}

#[async_trait]
impl ConfigInto<TimePoller> for TimePollerConfig {}

pub struct TimePoller {
    pub interval: Interval,
    pub ticks: u128,
}

#[async_trait]
impl FromConfig<TimePollerConfig> for TimePoller {
    async fn from_config(
        config: &TimePollerConfig,
    ) -> std::result::Result<TimePoller, Box<dyn std::error::Error>> {
        Ok(TimePoller {
            interval: tokio::time::interval(Duration::from_millis(config.period_in_millis)),
            ticks: config.ticks,
        })
    }
}

#[async_trait]
impl Poll<(), TimePollerConfig> for TimePoller {
    async fn poll(&mut self) -> Result<Option<()>, Box<dyn Error + Send + Sync>> {
        match self.ticks > 0 {
            true => self.ticks -= 1,
            false => return Ok(None),
        }
        self.interval.tick().await;
        Ok(Some(()))
    }
}

pub struct TimeListener {
    pub ticks: u128,
    pub period_in_millis: u64,
    pub senders: Vec<Arc<Sender<()>>>,
}

#[async_trait]
impl FromConfig<TimeListenerConfig> for TimeListener {
    async fn from_config(
        config: &TimeListenerConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(TimeListener {
            ticks: config.ticks,
            period_in_millis: config.period_in_millis,
            senders: vec![],
        })
    }
}

#[async_trait]
impl Listen<(), TimeListenerConfig> for TimeListener {
    async fn run(&mut self) {
        let mut ticks = self.ticks;
        let mut interval = tokio::time::interval(Duration::from_millis(self.period_in_millis));
        while ticks > 0 {
            interval.tick().await;
            let mut jhs: Vec<JoinHandle<()>> = vec![];
            for sender in self.senders.as_slice() {
                let tx = sender.clone();
                jhs.push(spawn_send!(tx, ()));
            }
            wait_join_handles!(jhs);
            ticks -= 1;
        }
    }

    async fn add_sender(&mut self, sender: Arc<Sender<()>>) {
        self.senders.push(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Listener, Poller};
    use crate::poller;
    use crate::source::timer::TimePollerConfig;
    use crate::FromFile;
    use crate::{channel, listener, spawn_join, Pipe, TimeListenerConfig};
    use tokio::sync::mpsc::{channel, Receiver};

    async fn on_receive(rx: &mut Receiver<()>, ticks: u128) {
        let mut i = 0;
        while ticks > i {
            rx.recv().await.unwrap();
            println!("tick: #{:#?}", i);
            i += 1;
        }
    }

    #[tokio::test]
    async fn test_time_poller() {
        let (tx, mut rx) = channel!((), 1024);
        let mut source = poller!(
            "timer",
            "resources/catalogs/timer.yml",
            TimePollerConfig,
            [tx]
        );
        spawn_join!(source);
        on_receive(&mut rx, 10).await;
    }

    #[tokio::test]
    async fn test_time_listener() {
        let (tx, mut rx) = channel!((), 1024);
        let mut listener = listener!(
            "timer",
            "resources/catalogs/timer.yml",
            TimeListenerConfig,
            [tx]
        );
        spawn_join!(listener);
        on_receive(&mut rx, 10).await;
    }
}
