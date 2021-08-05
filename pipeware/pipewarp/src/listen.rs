use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct WarpIngestionServerConfig {
    ip: String,
    port: u16,
}

impl FromPath for WarpIngestionServerConfig {}

impl ConfigInto<WarpIngestionServer> for WarpIngestionServerConfig {}

#[derive(Default)]
pub struct WarpIngestionServerState {
    pause: AtomicBool,
}

impl WarpIngestionServerState {
    pub fn is_pause(&self) -> bool {
        self.pause.load(Ordering::Acquire)
    }

    pub fn set_pause(&self, pause: bool) {
        self.pause.store(pause, Ordering::Release)
    }
}

pub struct WarpIngestionServer {
    socket: SocketAddr,
    tx: Option<Sender<Vec<u8>>>,
    state: Arc<WarpIngestionServerState>,
}

#[async_trait]
impl FromConfig<WarpIngestionServerConfig> for WarpIngestionServer {
    async fn from_config(config: WarpIngestionServerConfig) -> anyhow::Result<Self> {
        let ip_port = format!("{}:{}", config.ip, config.port);
        let socket: SocketAddr = ip_port.parse()?;
        Ok(WarpIngestionServer {
            socket,
            tx: None,
            state: Default::default(),
        })
    }
}

#[async_trait]
impl Listen<Vec<u8>, WarpIngestionServerConfig> for WarpIngestionServer {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await;
        Ok(())
    }

    fn set_sender(&mut self, sender: tokio::sync::mpsc::Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}

impl WarpIngestionServer {
    async fn do_run(&self) {
        let tx = self
            .tx
            .to_owned()
            .expect("sender not found for rest server as listener");
        let state = self.state.to_owned();
        let api = filters::ingest(tx, state);
        let socket = self.socket.to_owned();
        warp::serve(api).run(socket).await
    }
}

mod filters {
    use super::{handlers, WarpIngestionServerState};
    use std::sync::Arc;
    use tokio::sync::mpsc::Sender;
    use warp::Filter;

    pub fn ingest(
        sender: Sender<Vec<u8>>,
        state: Arc<WarpIngestionServerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        ingest_v1(sender, state.to_owned())
            .or(pause_v1(state.to_owned()))
            .or(resume_v1(state))
            .or(health())
    }

    pub fn ingest_v1(
        sender: Sender<Vec<u8>>,
        state: Arc<WarpIngestionServerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "ingest")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_sender(sender))
            .and(with_state(state))
            .and_then(handlers::send_data)
    }

    pub fn pause_v1(
        state: Arc<WarpIngestionServerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "pause")
            .and(warp::post())
            .and(with_state(state))
            .and_then(handlers::pause)
    }

    pub fn resume_v1(
        state: Arc<WarpIngestionServerState>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "resume")
            .and(warp::post())
            .and(with_state(state))
            .and_then(handlers::resume)
    }

    fn with_sender(
        sender: Sender<Vec<u8>>,
    ) -> impl Filter<Extract = (Sender<Vec<u8>>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || sender.clone())
    }

    fn with_state(
        state: Arc<WarpIngestionServerState>,
    ) -> impl Filter<Extract = (Arc<WarpIngestionServerState>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || state.clone())
    }

    pub fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "health")
            .and(warp::get())
            .and_then(handlers::health)
    }
}

mod handlers {
    use super::{models, WarpIngestionServerState};
    use std::{convert::Infallible, sync::Arc};
    use tokio::sync::mpsc::Sender;
    use warp::http::{Response, StatusCode};

    pub async fn send_data(
        bytes: bytes::Bytes,
        sender: Sender<Vec<u8>>,
        state: Arc<WarpIngestionServerState>,
    ) -> Result<impl warp::Reply, Infallible> {
        if state.is_pause() {
            let maintenance = models::Maintenance::new("ingestion server paused".to_string());
            return Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(serde_json::to_string(&maintenance).unwrap()));
        }
        let size = bytes.len();
        match sender.send(bytes.to_vec()).await {
            Ok(_) => {
                let success = models::Success::new(size);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(serde_json::to_string(&success).unwrap()))
            }
            Err(e) => {
                let failure = models::Failure::new(e.to_string());
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(serde_json::to_string(&failure).unwrap()))
            }
        }
    }

    pub async fn pause(
        state: Arc<WarpIngestionServerState>,
    ) -> Result<impl warp::Reply, Infallible> {
        state.set_pause(true);
        Ok(StatusCode::OK)
    }

    pub async fn resume(
        state: Arc<WarpIngestionServerState>,
    ) -> Result<impl warp::Reply, Infallible> {
        state.set_pause(false);
        Ok(StatusCode::OK)
    }

    pub async fn health() -> Result<impl warp::Reply, Infallible> {
        Ok(StatusCode::OK)
    }
}

mod models {

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Success {
        // byte size
        pub size: usize,
    }

    impl Success {
        pub fn new(size: usize) -> Self {
            Success { size }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Failure {
        pub error: String,
    }

    impl Failure {
        pub fn new(error: String) -> Self {
            Failure { error }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Maintenance {
        pub message: String,
    }

    impl Maintenance {
        pub fn new(message: String) -> Self {
            Maintenance { message }
        }
    }
}
