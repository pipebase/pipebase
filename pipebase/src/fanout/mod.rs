mod hash;
mod select;

use log::error;
use select::Select;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::{select_range_error, Result};
pub struct Hasher<'a, T, U> {
    name: &'a str,
    rx: Receiver<T>,
    txs: Vec<Sender<U>>,
    // p: Box<dyn Procedure<T, U>>,
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
}
