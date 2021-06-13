use async_trait::async_trait;
use log::error;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::{ConfigInto, FromConfig, FromFile, Listen};

#[derive(Deserialize)]
pub struct TimeListenerConfig {
    pub period_in_millis: u64,
    pub ticks: u128,
}

impl FromFile for TimeListenerConfig {}

#[async_trait]
impl ConfigInto<TimeListener> for TimeListenerConfig {}

pub struct TimeListener {
    pub ticks: u128,
    pub period_in_millis: u64,
    pub sender: Option<Arc<Sender<()>>>,
}

#[async_trait]
impl FromConfig<TimeListenerConfig> for TimeListener {
    async fn from_config(
        config: &TimeListenerConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(TimeListener {
            ticks: config.ticks,
            period_in_millis: config.period_in_millis,
            sender: None,
        })
    }
}

#[async_trait]
impl Listen<(), TimeListenerConfig> for TimeListener {
    async fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut ticks = self.ticks;
        let mut interval = tokio::time::interval(Duration::from_millis(self.period_in_millis));
        while ticks > 0 {
            interval.tick().await;
            match Self::send_data(self.sender.to_owned(), ()).await {
                true => (),
                false => break,
            }
            ticks -= 1;
        }
        Ok(())
    }

    async fn set_sender(&mut self, sender: Arc<Sender<()>>) {
        self.sender = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use crate::FromFile;
    use crate::{channel, listener, spawn_join, Listener, Pipe, TimeListenerConfig};
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

    #[tokio::test]
    async fn test_receiver_drop() {
        let (tx, rx) = channel!((), 1024);
        let mut listener = listener!(
            "timer",
            "resources/catalogs/timer.yml",
            TimeListenerConfig,
            [tx]
        );
        drop(rx);
        let start_millis = std::time::SystemTime::now();
        // start timer run 10 ticks each 1 second interval
        spawn_join!(listener);
        let now_millis = std::time::SystemTime::now();
        // listener should exit asap
        let duration = now_millis.duration_since(start_millis).unwrap();
        assert!(duration.as_secs() < 3)
    }
}
