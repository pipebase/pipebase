use async_trait::async_trait;
use log::error;
use log::info;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::channel;

#[async_trait]
pub trait Listen<T, C>: Send + Sync + FromConfig<C>
where
    T: Send + 'static,
{
    async fn run(&mut self) -> anyhow::Result<()>;
    async fn set_sender(&mut self, sender: Arc<Sender<T>>);
    // send data and return true if succeed
    async fn send_data(sender: Option<Arc<Sender<T>>>, t: T) -> bool {
        let sender = match sender {
            Some(sender) => sender,
            None => return false,
        };
        match sender.send(t).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

pub struct Listener<'a, T, L, C>
where
    T: Clone + Send + 'static,
    L: Listen<T, C> + 'static,
    C: ConfigInto<L> + Send + Sync,
{
    name: &'a str,
    config: C,
    txs: HashMap<usize, Arc<Sender<T>>>,
    listener: PhantomData<L>,
    context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T, L, C> Pipe<T> for Listener<'a, T, L, C>
where
    T: Clone + Send + 'static,
    L: Listen<T, C> + 'static,
    C: ConfigInto<L> + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        // connect listener
        let (tx, mut rx) = channel::<T>(1024);
        let mut listener = self.config.config_into().await.unwrap();
        // start listener
        let join_listener = tokio::spawn(async move {
            listener.set_sender(tx.into()).await;
            match listener.run().await {
                Ok(_) => info!("listener exit ..."),
                Err(e) => error!("listenr exit with error {}", e),
            };
        });
        // start event loop
        let mut txs = self.txs.to_owned();
        let context = self.context.clone();
        let join_event_loop = tokio::spawn(async move {
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
                let t = match rx.recv().await {
                    Some(t) => t,
                    None => {
                        Self::inc_success_run(&context).await;
                        break;
                    }
                };
                Self::set_state(&context, State::Send).await;
                let mut jhs = HashMap::new();
                for (idx, tx) in &txs {
                    let u_clone: T = t.to_owned();
                    jhs.insert(idx.to_owned(), Self::spawn_send(tx.clone(), u_clone));
                }
                let drop_sender_indices = Self::wait_join_handles(jhs).await;
                Self::filter_senders_by_indices(&mut txs, drop_sender_indices);
                Self::inc_success_run(&context).await;
            }
            Self::set_state(&context, State::Done).await;
        });
        // join listener and loop
        match tokio::spawn(async move { tokio::join!(join_listener, join_event_loop) }).await {
            Ok(_) => (),
            Err(err) => {
                error!("listener join error {:#?}", err)
            }
        }
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<T>) {
        let idx = self.txs.len();
        self.txs.insert(idx, Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

impl<'a, T, L, C> Listener<'a, T, L, C>
where
    T: Clone + Send + 'static,
    L: Listen<T, C> + 'static,
    C: ConfigInto<L> + Send + Sync,
{
    pub fn new(name: &'a str, config: C) -> Self {
        Listener {
            name: name,
            config: config,
            txs: HashMap::new(),
            listener: std::marker::PhantomData,
            context: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! listener {
    (
        $name:expr, $path:expr, $config:ty, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_path($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Listener::new($name, config);
            $(
                pipe.add_sender($tx);
            )*
            pipe
        }
    };
    (
        $name:expr, $path:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        listener!($name, $path, $config, [$( $tx ), *])
    };
    (
        $name:expr, $config:ty, $rx:expr, [$( $tx:expr ), *]
    ) => {
        listener!($name, "", $config, [$( $tx ), *])
    };
    (
        $name:expr, $config:ty, $rx:expr
    ) => {
        listener!($name, "", $config, [$( $tx ), *])
    };
}
