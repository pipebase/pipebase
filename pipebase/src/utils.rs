use crate::{Context, State};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

// Sender Operations
pub(crate) fn senders_as_map<U>(txs: Vec<Sender<U>>) -> HashMap<usize, Sender<U>> {
    let mut i: usize = 0;
    let mut txs_map: HashMap<usize, Sender<U>> = HashMap::new();
    for tx in txs {
        txs_map.insert(i, tx);
        i += 1;
    }
    txs_map
}

pub(crate) fn spawn_send<U>(
    tx: Sender<U>,
    t: U,
) -> JoinHandle<core::result::Result<(), SendError<U>>>
where
    U: Send + 'static,
{
    tokio::spawn(async move {
        match tx.send(t).await {
            Ok(()) => Ok(()),
            Err(err) => {
                log::error!("selector send error {}", err.to_string());
                Err(err)
            }
        }
    })
}

pub(crate) async fn wait_join_handles<U>(
    join_handles: HashMap<usize, JoinHandle<core::result::Result<(), SendError<U>>>>,
) -> Vec<usize> {
    let mut drop_sender_indices = Vec::new();
    for (idx, jh) in join_handles {
        let result = match jh.await {
            Ok(res) => res,
            Err(err) => {
                log::error!("join error in pipe err: {:#?}", err);
                drop_sender_indices.push(idx);
                continue;
            }
        };
        match result {
            Ok(()) => (),
            Err(err) => {
                log::error!("send error {}", err);
                drop_sender_indices.push(idx);
            }
        }
    }
    drop_sender_indices
}

pub(crate) fn filter_senders_by_indices<U>(
    senders: &mut HashMap<usize, Sender<U>>,
    remove_indices: Vec<usize>,
) {
    for idx in remove_indices {
        senders.remove(&idx);
    }
}

// Context Operations
pub(crate) async fn set_state(context: &Arc<RwLock<Context>>, state: State) {
    let mut ctx = context.write().await;
    ctx.set_state(state)
}

pub(crate) async fn inc_total_run(context: &Arc<RwLock<Context>>) {
    let mut ctx = context.write().await;
    ctx.inc_total_run()
}

pub(crate) async fn inc_success_run(context: &Arc<RwLock<Context>>) {
    let mut ctx = context.write().await;
    ctx.inc_success_run()
}

// Test Operations
pub(crate) async fn populate_records<T, U>(tx: Sender<T>, records: U)
where
    U: IntoIterator<Item = T>,
{
    for record in records {
        let _ = tx.send(record).await;
    }
}
