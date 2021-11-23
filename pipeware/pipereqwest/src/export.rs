use crate::client::{BasicAuth, ReqwestClient};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use reqwest::{Body, Response};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

#[derive(Deserialize)]
pub struct ReqwestPosterConfig {
    url: String,
    basic_auth: Option<BasicAuth>,
    bear_auth_token: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl FromPath for ReqwestPosterConfig {}

impl ConfigInto<ReqwestPoster> for ReqwestPosterConfig {}

pub struct ReqwestPoster {
    client: ReqwestClient,
}

#[async_trait]
impl FromConfig<ReqwestPosterConfig> for ReqwestPoster {
    async fn from_config(config: ReqwestPosterConfig) -> anyhow::Result<Self> {
        Ok(ReqwestPoster {
            client: ReqwestClient::new(
                config.url,
                config.basic_auth,
                config.bear_auth_token,
                config.headers.unwrap_or_default(),
            )?,
        })
    }
}

#[async_trait]
impl<T> Export<T, ReqwestPosterConfig> for ReqwestPoster
where
    T: Into<Body> + Send + 'static,
{
    async fn export(&mut self, body: T) -> anyhow::Result<()> {
        let resp = self.client.post(body).await?;
        Self::log_response(resp).await;
        Ok(())
    }
}

impl ReqwestPoster {
    async fn log_response(resp: Response) {
        info!("response status: {}", resp.status());
        if let Ok(text) = resp.text().await {
            info!("response text: {}", text)
        }
    }
}
