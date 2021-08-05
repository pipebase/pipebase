use async_trait::async_trait;
use std::{collections::HashMap, net::SocketAddr};

use pipebase::common::{ConfigInto, Context, FromConfig, FromPath, PipeContext};
use pipebase::context::StoreContext;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::channel;

#[derive(Deserialize, Serialize)]
pub struct PipeContextQuery {
    state: String,
}

#[derive(Deserialize)]
pub struct WarpContextServerConfig {
    ip: String,
    port: u16,
}

impl FromPath for WarpContextServerConfig {}

#[async_trait]
impl ConfigInto<WarpContextServer> for WarpContextServerConfig {}

pub struct WarpContextServer {
    socket: SocketAddr,
    contexts: HashMap<String, std::sync::Arc<Context>>,
}

#[async_trait]
impl FromConfig<WarpContextServerConfig> for WarpContextServer {
    async fn from_config(config: WarpContextServerConfig) -> anyhow::Result<Self> {
        let ip_port = format!("{}:{}", config.ip, config.port);
        let socket: SocketAddr = ip_port.parse()?;
        Ok(WarpContextServer {
            socket,
            contexts: HashMap::new(),
        })
    }
}

#[async_trait]
impl StoreContext<WarpContextServerConfig> for WarpContextServer {
    fn store_context(&mut self, pipe_name: String, context: std::sync::Arc<Context>) {
        self.contexts.insert(pipe_name, context);
    }

    fn load_context(&self, pipe_name: &str) -> Option<&std::sync::Arc<Context>> {
        self.contexts.get(pipe_name)
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await;
        Ok(())
    }
}

impl WarpContextServer {
    async fn do_run(&self) {
        let repository = self.repository();
        let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);
        let api = filters::contexts(repository, shutdown_tx);
        let socket = self.socket.to_owned();
        let (_, server) = warp::serve(api).bind_with_graceful_shutdown(socket, async move {
            shutdown_rx.recv().await;
        });
        server.await;
    }

    fn repository(&self) -> PipeContextRepository {
        PipeContextRepository::new(self.contexts.to_owned())
    }
}

#[derive(Clone)]
pub struct PipeContextRepository {
    contexts: HashMap<String, std::sync::Arc<Context>>,
}

impl PipeContextRepository {
    fn new(contexts: HashMap<String, std::sync::Arc<Context>>) -> Self {
        PipeContextRepository { contexts }
    }

    fn get_context(&self, name: &str) -> Option<PipeContext> {
        let context = match self.contexts.get(name) {
            Some(context) => context,
            None => return None,
        };
        Some(PipeContext::new(
            name.to_owned(),
            context.get_state(),
            context.get_total_run(),
            context.get_failure_run(),
        ))
    }

    fn query_context(&self, query: PipeContextQuery) -> Vec<PipeContext> {
        let contexts: Vec<PipeContext> = self
            .contexts
            .iter()
            .map(|(name, context)| {
                PipeContext::new(
                    name.to_owned(),
                    context.get_state(),
                    context.get_total_run(),
                    context.get_failure_run(),
                )
            })
            .filter(|ctx| ctx.get_state() == &query.state)
            .collect();
        contexts
    }

    fn list_contexts(&self) -> Vec<PipeContext> {
        let contexts: Vec<PipeContext> = self
            .contexts
            .iter()
            .map(|(name, context)| {
                PipeContext::new(
                    name.to_owned(),
                    context.get_state(),
                    context.get_total_run(),
                    context.get_failure_run(),
                )
            })
            .collect();
        contexts
    }
}

mod filters {
    use crate::PipeContextQuery;

    use super::handlers;
    use super::PipeContextRepository;
    use tokio::sync::mpsc::Sender;
    use warp::Filter;

    pub fn contexts(
        repository: PipeContextRepository,
        shutdown_tx: Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        context_query_v1(repository.to_owned())
            .or(context_get_v1(repository.to_owned()))
            .or(context_list_v1(repository))
            .or(shutdown_v1(shutdown_tx))
            .or(health())
    }

    pub fn context_list_v1(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "pipe")
            .and(warp::get())
            .and(with_repository(repository))
            .and_then(handlers::list_contexts)
    }

    pub fn context_get_v1(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "pipe" / String)
            .and(warp::get())
            .and(with_repository(repository))
            .and_then(handlers::get_context)
    }

    pub fn context_query_v1(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "pipe")
            .and(warp::get())
            .and(warp::query::<PipeContextQuery>())
            .and(with_repository(repository))
            .and_then(handlers::query_context)
    }

    pub fn shutdown_v1(
        shutdown_tx: Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "shutdown")
            .and(warp::post())
            .and(with_shutdown_tx(shutdown_tx))
            .and_then(handlers::shutdown)
    }

    fn with_repository(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = (PipeContextRepository,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || repository.clone())
    }

    fn with_shutdown_tx(
        shutdown_tx: Sender<()>,
    ) -> impl Filter<Extract = (Sender<()>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || shutdown_tx.clone())
    }

    pub fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("v1" / "health")
            .and(warp::get())
            .and_then(handlers::health)
    }
}

mod handlers {
    use super::{models, PipeContextQuery, PipeContextRepository};
    use std::convert::Infallible;
    use tokio::sync::mpsc::Sender;
    use warp::http::{Response, StatusCode};

    pub async fn list_contexts(
        repository: PipeContextRepository,
    ) -> Result<impl warp::Reply, Infallible> {
        let contexts = repository.list_contexts();
        Ok(warp::reply::json(&contexts))
    }

    pub async fn get_context(
        name: String,
        repository: PipeContextRepository,
    ) -> Result<impl warp::Reply, Infallible> {
        match repository.get_context(&name) {
            Some(context) => Ok(Response::builder()
                .status(StatusCode::OK)
                .body(serde_json::to_string(&context).unwrap())),
            None => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(format!("pipe '{}' not found", &name))),
        }
    }

    pub async fn query_context(
        query: PipeContextQuery,
        repository: PipeContextRepository,
    ) -> Result<impl warp::Reply, Infallible> {
        let contexts = repository.query_context(query);
        Ok(warp::reply::json(&contexts))
    }

    pub async fn shutdown(shutdown_tx: Sender<()>) -> Result<impl warp::Reply, Infallible> {
        let sent = shutdown_tx.send(()).await.is_ok();
        if sent {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body("shutdown ...".to_string()));
        }
        let failure = models::Failure::new("failed to shutdown".to_string());
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(serde_json::to_string(&failure).unwrap()))
    }

    pub async fn health() -> Result<impl warp::Reply, Infallible> {
        Ok(StatusCode::OK)
    }
}

mod models {

    use serde::{Deserialize, Serialize};

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
