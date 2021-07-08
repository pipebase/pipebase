use async_trait::async_trait;
use std::{collections::HashMap, net::SocketAddr};

use pipebase::{ConfigInto, FromConfig, FromPath, PipeContext, StoreContext};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PipeContextQuery {
    state: String,
}

#[derive(Deserialize)]
pub struct ContextRestServerConfig {
    ip: String,
    port: u16,
}

impl FromPath for ContextRestServerConfig {}

#[async_trait]
impl ConfigInto<ContextRestServer> for ContextRestServerConfig {}

pub struct ContextRestServer {
    socket: SocketAddr,
    contexts: HashMap<String, std::sync::Arc<pipebase::Context>>,
}

#[async_trait]
impl FromConfig<ContextRestServerConfig> for ContextRestServer {
    async fn from_config(config: &ContextRestServerConfig) -> anyhow::Result<Self> {
        let ip_port = format!("{}:{}", config.ip, config.port);
        let socket: SocketAddr = ip_port.parse()?;
        Ok(ContextRestServer {
            socket: socket,
            contexts: HashMap::new(),
        })
    }
}

#[async_trait]
impl StoreContext<ContextRestServerConfig> for ContextRestServer {
    fn store_context(&mut self, pipe_name: String, context: std::sync::Arc<pipebase::Context>) {
        self.contexts.insert(pipe_name, context);
    }

    fn load_context(&self, pipe_name: &str) -> Option<&std::sync::Arc<pipebase::Context>> {
        self.contexts.get(pipe_name)
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await;
        Ok(())
    }
}

impl ContextRestServer {
    async fn do_run(&self) {
        let repository = self.repository();
        let api = filters::contexts(repository);
        let socket = self.socket.to_owned();
        warp::serve(api).run(socket).await
    }

    fn repository(&self) -> PipeContextRepository {
        PipeContextRepository::new(self.contexts.to_owned())
    }
}

#[derive(Clone)]
pub struct PipeContextRepository {
    contexts: HashMap<String, std::sync::Arc<pipebase::Context>>,
}

impl PipeContextRepository {
    fn new(contexts: HashMap<String, std::sync::Arc<pipebase::Context>>) -> Self {
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
    use warp::Filter;

    pub fn contexts(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        context_list_v1(repository.to_owned())
            .or(context_get_v1(repository.to_owned()))
            .or(context_query_v1(repository.to_owned()))
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

    fn with_repository(
        repository: PipeContextRepository,
    ) -> impl Filter<Extract = (PipeContextRepository,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || repository.clone())
    }
}

mod handlers {
    use super::{PipeContextQuery, PipeContextRepository};
    use std::convert::Infallible;
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
                .body(format!(r#"pipe {} not found"#, &name))),
        }
    }

    pub async fn query_context(
        query: PipeContextQuery,
        repository: PipeContextRepository,
    ) -> Result<impl warp::Reply, Infallible> {
        let contexts = repository.query_context(query);
        Ok(warp::reply::json(&contexts))
    }
}
