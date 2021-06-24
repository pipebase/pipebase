use async_trait::async_trait;
use log::error;
use log::info;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::channel;

#[async_trait]
pub trait Listen<T: Send + 'static, C>: Send + Sync + FromConfig<C> {
    async fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
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

pub struct Listener<'a, T: Send + 'static, L: Listen<T, C>, C: ConfigInto<L>> {
    pub name: &'a str,
    pub txs: Vec<Arc<Sender<T>>>,
    pub config: C,
    pub listener: PhantomData<L>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Clone + Send + 'static, L: Listen<T, C> + 'static, C: ConfigInto<L> + Send + Sync>
    Pipe<T> for Listener<'a, T, L, C>
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
                let mut jhs = vec![];
                for tx in txs.as_slice() {
                    let u_clone: T = t.to_owned();
                    jhs.push(Self::spawn_send(tx.clone(), u_clone));
                }
                let dropped_receiver_idxs = Self::wait_join_handles(jhs).await;
                txs = Self::filter_sender_by_dropped_receiver_idx(
                    txs.to_owned(),
                    dropped_receiver_idxs,
                );
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
        self.txs.push(Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! listener {
    (
        $name:expr, $path:expr, $config:ty, [$( $tx:expr ), *]
    ) => {
        {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Listener {
                name: $name,
                txs: vec![],
                config: config,
                listener: std::marker::PhantomData,
                context: Default::default()
            };
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
