use crate::context::{Context, State};
use async_trait::async_trait;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

use crate::Pipe;

pub struct Streamer<'a, T, U>
where
    T: IntoIterator<Item = U> + Send,
    U: Clone + Send + 'static,
{
    name: &'a str,
    rx: Receiver<T>,
    txs: HashMap<usize, Arc<Sender<U>>>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, U> Pipe<U> for Streamer<'a, T, U>
where
    T: IntoIterator<Item = U> + Send,
    U: Clone + Send + 'static,
{
    async fn run(&mut self) -> crate::error::Result<()> {
        log::info!("streamer {} run ...", self.name);
        loop {
            Self::inc_total_run(&self.context).await;
            Self::set_state(&self.context, State::Receive).await;
            // if all receiver dropped, sender drop as well
            match self.txs.is_empty() {
                true => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
                false => (),
            }
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => {
                    Self::inc_success_run(&self.context).await;
                    break;
                }
            };
            Self::set_state(&self.context, State::Send).await;
            // avoid `Send` is not implemented for `< as IntoIterator>::IntoIter
            let items: Vec<U> = Vec::from_iter(t);
            for item in items {
                let mut jhs = HashMap::new();
                for (idx, tx) in &self.txs {
                    let item_clone = item.to_owned();
                    jhs.insert(idx.to_owned(), Self::spawn_send(tx.to_owned(), item_clone));
                }
                let drop_sender_indices = Self::wait_join_handles(jhs).await;
                Self::filter_senders_by_indices(&mut self.txs, drop_sender_indices);
            }
        }
        log::info!("streamer {} exit ...", self.name);
        Self::set_state(&self.context, State::Done).await;
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<U>) {
        let idx = self.txs.len();
        self.txs.insert(idx, Arc::new(tx));
    }

    fn get_context(&self) -> Arc<tokio::sync::RwLock<crate::Context>> {
        self.context.to_owned()
    }
}

impl<'a, T, U> Streamer<'a, T, U>
where
    T: IntoIterator<Item = U> + Send,
    U: Clone + Send + 'static,
{
    pub fn new(name: &'a str, rx: Receiver<T>) -> Self {
        Streamer {
            name: name,
            rx: rx,
            txs: HashMap::new(),
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! streamer {
    (
        $name:expr, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let mut pipe = Streamer::new($name, $rx);
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        streamer!($name, $rx, [$( $tx ), *])
    };
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        streamer!($name, $rx, [$( $tx ), *])
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    use std::collections::HashMap;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<HashMap<String, u32>>, records: HashMap<String, u32>) {
        let _ = tx.send(records).await;
    }

    #[tokio::test]
    async fn test_streamer() {
        let (tx0, rx0) = channel!(HashMap<String, u32>, 1024);
        let (tx1, mut rx1) = channel!((String, u32), 1024);
        let mut pipe = streamer!("tuple_streamer", rx0, [tx1]);
        let mut records: HashMap<String, u32> = HashMap::new();
        records.insert("one".to_owned(), 1);
        records.insert("two".to_owned(), 2);
        let f0 = populate_records(tx0, records);
        f0.await;
        spawn_join!(pipe);
        let mut records: HashMap<String, u32> = HashMap::new();
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        let (left, right) = rx1.recv().await.unwrap();
        records.insert(left, right);
        assert_eq!(&1, records.get("one").unwrap());
        assert_eq!(&2, records.get("two").unwrap())
    }
}
