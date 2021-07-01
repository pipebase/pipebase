mod file;
mod iterator;

pub use file::*;
pub use iterator::*;

use crate::context::{Context, State};
use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};

use crate::{ConfigInto, FromConfig, Pipe};

#[async_trait]
pub trait Stream<T, U, C>: Send + Sync + FromConfig<C>
where
    U: Send + 'static,
{
    async fn stream(&mut self, t: T) -> anyhow::Result<()>;
    fn set_sender(&mut self, sender: Sender<U>);
}

pub struct Streamer<'a, T, U, S, C>
where
    T: Send,
    U: Clone + Send + 'static,
    S: Stream<T, U, C>,
    C: ConfigInto<S> + Send + Sync,
{
    name: &'a str,
    config: C,
    rx: Arc<Mutex<Receiver<T>>>,
    txs: HashMap<usize, Arc<Sender<U>>>,
    streamer: PhantomData<S>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, U, S, C> Pipe<T, U, S, C> for Streamer<'a, T, U, S, C>
where
    T: Send + 'static,
    U: Clone + Send + 'static,
    S: Stream<T, U, C> + 'static,
    C: ConfigInto<S> + Send + Sync,
{
    async fn run(&mut self) -> crate::error::Result<()> {
        let (tx0, mut rx0) = channel::<U>(1024);
        let mut streamer = self.config.config_into().await.unwrap();
        streamer.set_sender(tx0);
        let rx = self.rx.to_owned();
        let name = self.name.to_owned();
        let streamer_loop = tokio::spawn(async move {
            let mut rx = rx.lock().await;
            log::info!("streamer {} run ...", name);
            loop {
                let t = match (*rx).recv().await {
                    Some(t) => t,
                    None => break,
                };
                match streamer.stream(t).await {
                    Ok(_) => continue,
                    Err(err) => {
                        log::error!("streamer error {}", err);
                        break;
                    }
                }
            }
            log::info!("streamer {} exit ...", name);
        });
        let mut txs = self.txs.to_owned();
        let context = self.context.clone();
        let sender_loop = tokio::spawn(async move {
            loop {
                Self::inc_total_run(&context).await;
                Self::set_state(&context, State::Receive).await;
                // if all receiver dropped, sender drop as well
                match txs.is_empty() {
                    true => {
                        Self::inc_success_run(&context).await;
                        break;
                    }
                    false => (),
                }
                let u = match rx0.recv().await {
                    Some(u) => u,
                    None => {
                        Self::inc_success_run(&context).await;
                        // streamer loop break
                        break;
                    }
                };
                Self::set_state(&context, State::Send).await;
                let mut jhs = HashMap::new();
                for (idx, tx) in &txs {
                    let u_clone: U = u.to_owned();
                    jhs.insert(idx.to_owned(), Self::spawn_send(tx.clone(), u_clone));
                }
                let drop_sender_indices = Self::wait_join_handles(jhs).await;
                Self::filter_senders_by_indices(&mut txs, drop_sender_indices);
                Self::inc_success_run(&context).await;
            }
            Self::set_state(&context, State::Done).await;
        });
        // join listener and loop
        match tokio::spawn(async move { tokio::join!(streamer_loop, sender_loop) }).await {
            Ok(_) => (),
            Err(err) => {
                log::error!("streamer join error {:#?}", err)
            }
        }
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

impl<'a, T, U, S, C> Streamer<'a, T, U, S, C>
where
    T: Send,
    U: Clone + Send + 'static,
    S: Stream<T, U, C>,
    C: ConfigInto<S> + Send + Sync,
{
    pub fn new(name: &'a str, config: C, rx: Receiver<T>) -> Self {
        Streamer {
            name: name,
            config: config,
            rx: Arc::new(Mutex::new(rx)),
            txs: HashMap::new(),
            streamer: PhantomData,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! streamer {
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Streamer::new($name, config, $rx);
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        streamer!($name, "", $config, $rx, [$( $tx ), *])
    };
}
