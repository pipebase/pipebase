mod default;

pub use default::*;

use crate::{ConfigInto, FromConfig, Pipe};
use async_trait::async_trait;
use std::hash::Hash;
use std::marker::PhantomData;

use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

use crate::context::{Context, State};
use crate::error::{select_range_error, Result};

pub trait HashSelect<T: Hash, U>: Send + Sync + FromConfig<U> {
    fn select(&mut self, t: &T) -> Vec<usize>;
    fn get_range(&mut self) -> usize;
}

pub struct HashSelector<'a, T: Hash, S: HashSelect<T, C>, C: ConfigInto<S>> {
    pub name: &'a str,
    pub rx: Receiver<T>,
    pub txs: Vec<Arc<Sender<T>>>,
    pub config: C,
    pub selector: PhantomData<S>,
    pub context: Arc<RwLock<Context>>,
}

#[async_trait]
impl<'a, T: Clone + Hash + Send + 'static, S: HashSelect<T, C>, C: ConfigInto<S> + Send + Sync>
    Pipe<T> for HashSelector<'a, T, S, C>
{
    async fn run(&mut self) -> Result<()> {
        let mut selector = self.config.config_into().await.unwrap();
        let selector_range = selector.get_range();
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
            Self::inc_total_run(self.context.clone()).await;
            Self::set_state(self.context.clone(), State::Receive).await;
            let t = self.rx.recv().await;
            let t = match t {
                Some(t) => t,
                None => break,
            };
            Self::set_state(self.context.clone(), State::Send).await;
            let mut jhs = vec![];
            for i in selector.select(&t) {
                let tx = self.txs.get(i).unwrap().to_owned();
                let t_clone = t.to_owned();
                jhs.push(Self::spawn_send(tx, t_clone));
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

    fn add_sender(&mut self, tx: Sender<T>) {
        self.txs.push(Arc::new(tx));
    }

    fn get_context(&self) -> Arc<RwLock<Context>> {
        self.context.clone()
    }
}

#[macro_export]
macro_rules! hselector {
    (
        $name:expr, $path:expr, $config:ty, $rx: ident, [$( $sender:ident ), *]
    ) => {
        async move {
            let config = <$config>::from_file($path).expect(&format!("invalid config file location {}", $path));
            let mut pipe = HashSelector {
                name: $name,
                rx: $rx,
                txs: vec![],
                config: config,
                selector: std::marker::PhantomData,
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
