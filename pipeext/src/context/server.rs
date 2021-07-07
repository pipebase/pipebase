use async_trait::async_trait;
use std::{collections::HashMap, net::SocketAddr};

use pipebase::{ConfigInto, FromConfig, FromPath, PipeContext, StoreContext};
use serde::Deserialize;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

const PIPE_CONTEXT_QUERY_PARAMETER_KEY: &'static str = "name";

#[derive(Deserialize)]
pub struct ContextRestServerConfig {
    ip: String,
    port: u16,
    path: String,
}

impl FromPath for ContextRestServerConfig {}

#[async_trait]
impl ConfigInto<ContextRestServer> for ContextRestServerConfig {}

pub struct ContextRestServer {
    socket: SocketAddr,
    path: String,
    contexts: HashMap<String, std::sync::Arc<pipebase::Context>>,
}

#[async_trait]
impl FromConfig<ContextRestServerConfig> for ContextRestServer {
    async fn from_config(config: &ContextRestServerConfig) -> anyhow::Result<Self> {
        let ip_port = format!("{}:{}", config.ip, config.port);
        let socket: SocketAddr = ip_port.parse()?;
        Ok(ContextRestServer {
            socket: socket,
            path: config.path.to_owned(),
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
        let path = self.path.to_owned();
        let repository = self.repository();
        let get_context = warp::get()
            .and(warp::path(path))
            .and(warp::query::<HashMap<String, String>>())
            .map(
                move |p: HashMap<String, String>| match p.get(PIPE_CONTEXT_QUERY_PARAMETER_KEY) {
                    Some(name) => match repository.get_context(name) {
                        Some(ref context) => Response::builder()
                            .status(StatusCode::OK)
                            .body(serde_json::to_string(context).unwrap()),
                        None => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(format!(r#"pipe {} not found"#, name)),
                    },
                    None => Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(format!(
                            r#"no "{}" param in query "#,
                            PIPE_CONTEXT_QUERY_PARAMETER_KEY
                        )),
                },
            );
        let socket = self.socket.to_owned();
        warp::serve(get_context).run(socket).await
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
}
