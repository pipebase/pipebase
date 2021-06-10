mod hash;
mod select;

use std::hash::Hash;

use hash::HashSelect;
use log::error;
use select::Select;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::{select_range_error, Result};
pub struct HashSelector<'a, T: Hash> {
    name: &'a str,
    rx: Receiver<T>,
    txs: Vec<Sender<T>>,
    selector: Box<dyn HashSelect<T>>,
}

impl<'a, T: Clone + Hash> HashSelector<'a, T> {
    pub async fn run(&mut self) -> Result<()> {
        let selector_range = self.selector.get_range();
        let sender_range = self.txs.len();
        match selector_range == sender_range {
            false => {
                return Err(select_range_error(&format!(
                    "selector/sender range not equal {} != {}",
                    selector_range, sender_range
                )))
            }
            _ => (),
        }
        loop {
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            for i in self.selector.select(&t) {
                let tx = self.txs.get(i).unwrap();
                match tx.send(t.to_owned()).await {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("selector send error {}", err.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(tx);
    }
}

pub struct Selector<'a, T> {
    name: &'a str,
    rx: Receiver<T>,
    txs: Vec<Sender<T>>,
    selector: Box<dyn Select>,
}

impl<'a, T: Clone> Selector<'a, T> {
    pub async fn run(&mut self) -> Result<()> {
        let selector_range = self.selector.get_range();
        let sender_range = self.txs.len();
        match selector_range == sender_range {
            false => {
                return Err(select_range_error(&format!(
                    "selector/sender range not equal {} != {}",
                    selector_range, sender_range
                )))
            }
            _ => (),
        }
        loop {
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            for i in self.selector.select() {
                let tx = self.txs.get(i).unwrap();
                match tx.send(t.to_owned()).await {
                    Ok(_) => continue,
                    Err(err) => {
                        error!("selector send error {}", err.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(tx);
    }
}

#[macro_export]
macro_rules! selector {
    (
        $name:expr, $path:expr, $config:ty, $select:ty, $rx: ident, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let selector = <$select>::from_config(&config).await.unwrap();
            let mut pipe = Selector {
                name: $name,
                rx: $rx,
                txs: vec![],
                selector: Box::new(selector),
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}

#[macro_export]
macro_rules! hselector {
    (
        $name:expr, $path:expr, $config:ty, $select:ty, $rx: ident, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let selector = <$select>::from_config(&config).await.unwrap();
            let mut pipe = HashSelector {
                name: $name,
                rx: $rx,
                txs: vec![],
                selector: Box::new(selector),
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}
