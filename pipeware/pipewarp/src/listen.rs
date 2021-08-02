use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct WarpIngestionServerConfig {
    ip: String,
    port: u16,
}

impl FromPath for WarpIngestionServerConfig {}

impl ConfigInto<WarpIngestionServer> for WarpIngestionServerConfig {}

pub struct WarpIngestionServer {
    socket: SocketAddr,
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FromConfig<WarpIngestionServerConfig> for WarpIngestionServer {
    async fn from_config(config: WarpIngestionServerConfig) -> anyhow::Result<Self> {
        let ip_port = format!("{}:{}", config.ip, config.port);
        let socket: SocketAddr = ip_port.parse()?;
        Ok(WarpIngestionServer { socket, tx: None })
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
        let api = filters::ingest(tx);
        let socket = self.socket.to_owned();
        warp::serve(api).run(socket).await
    }
}

mod filters {
    use super::handlers;
    use tokio::sync::mpsc::Sender;
    use warp::Filter;

    pub fn ingest(
        sender: Sender<Vec<u8>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        ingest_v1(sender).or(health())
    }

    pub fn ingest_v1(
        sender: Sender<Vec<u8>>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "ingest")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_sender(sender))
            .and_then(handlers::send_data)
    }

    fn with_sender(
        sender: Sender<Vec<u8>>,
    ) -> impl Filter<Extract = (Sender<Vec<u8>>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || sender.clone())
    }

    pub fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "health")
            .and(warp::get())
            .and_then(handlers::health)
    }
}

mod handlers {
    use super::models;
    use std::convert::Infallible;
    use tokio::sync::mpsc::Sender;
    use warp::http::{Response, StatusCode};

    pub async fn send_data(
        bytes: bytes::Bytes,
        sender: Sender<Vec<u8>>,
    ) -> Result<impl warp::Reply, Infallible> {
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
}
