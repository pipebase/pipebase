mod echo;
mod field;
mod filter;
mod project;

use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::context::{Context, State};
use crate::error::Result;
use crate::{ConfigInto, FromConfig, Pipe};
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Procedure<T, U, C>: Send + Sync + FromConfig<C> {
    async fn process(&mut self, data: &T) -> std::result::Result<U, Box<dyn std::error::Error>>;
}

pub struct Process<'a, T, U, P: Procedure<T, U, C>, C: ConfigInto<P>> {
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub txs: Vec<Arc<Sender<U>>>,
    pub config: C,
    pub procedure: PhantomData<P>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<
        'a,
        T: Send + Sync,
        U: Clone + Debug + Send + 'static,
        P: Procedure<T, U, C>,
        C: ConfigInto<P> + Send + Sync,
    > Pipe<U> for Process<'a, T, U, P, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut procedure = self.config.config_into().await.unwrap();
        loop {
            Self::inc_total_run(self.context.clone()).await;
            Self::set_state(self.context.clone(), State::Receive).await;
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            Self::set_state(self.context.clone(), State::Process).await;
            let u = match procedure.process(&t).await {
                Ok(u) => u,
                Err(e) => {
                    error!("process {} error {}", self.name, e);
                    continue;
                }
            };
            Self::set_state(self.context.clone(), State::Send).await;
            let mut jhs = vec![];
            for tx in self.txs.to_owned() {
                let u_clone: U = u.to_owned();
                jhs.push(Self::spawn_send(tx, u_clone));
            }
            match Self::wait_join_handles(jhs).await {
                _ => (),
            }
            Self::inc_success_run(self.context.clone()).await;
        }
        Self::set_state(self.context.clone(), State::Done).await;
        Self::inc_success_run(self.context.clone()).await;
        Ok(())
    }

    fn add_sender(&mut self, tx: Sender<U>) {
        self.txs.push(Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! process {
    (
        $name:expr, $path:expr, $config:ty, $rx: ident, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = Process {
                name: $name,
                rx: $rx,
                txs: vec![],
                config: config,
                procedure: std::marker::PhantomData,
                context: Default::default()
            };
            $(
                pipe.add_sender($sender);
            )*
            pipe
        }
        .await
    };
}
