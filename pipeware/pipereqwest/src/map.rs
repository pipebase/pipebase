use crate::client::{BasicAuth, ReqwestClient};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Render},
    map::Map,
};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ReqwestQueryConfig {
    base_url: String,
    basic_auth: Option<BasicAuth>,
    bear_auth_token: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl FromPath for ReqwestQueryConfig {}

impl ConfigInto<ReqwestQuery> for ReqwestQueryConfig {}

pub struct ReqwestQuery {
    client: ReqwestClient,
}

#[async_trait]
impl FromConfig<ReqwestQueryConfig> for ReqwestQuery {
    async fn from_config(config: ReqwestQueryConfig) -> anyhow::Result<Self> {
        Ok(ReqwestQuery {
            client: ReqwestClient::new(
                config.base_url,
                config.basic_auth,
                config.bear_auth_token,
                config.headers.unwrap_or_default(),
            )?,
        })
    }
}

#[async_trait]
impl<T> Map<Option<T>, Vec<u8>, ReqwestQueryConfig> for ReqwestQuery
where
    T: Serialize + Send + 'static,
{
    async fn map(&mut self, data: Option<T>) -> anyhow::Result<Vec<u8>> {
        self.query(data).await
    }
}

impl ReqwestQuery {
    async fn query<Q>(&self, query: Option<Q>) -> anyhow::Result<Vec<u8>>
    where
        Q: Serialize,
    {
        let resp = self.client.query(query).await?;
        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[derive(Deserialize)]
pub struct ReqwestGetterConfig {
    base_url: String,
    basic_auth: Option<BasicAuth>,
    bear_auth_token: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl FromPath for ReqwestGetterConfig {}

impl ConfigInto<ReqwestGetter> for ReqwestGetterConfig {}

pub struct ReqwestGetter {
    client: ReqwestClient,
}

#[async_trait]
impl FromConfig<ReqwestGetterConfig> for ReqwestGetter {
    async fn from_config(config: ReqwestGetterConfig) -> anyhow::Result<Self> {
        Ok(ReqwestGetter {
            client: ReqwestClient::new(
                config.base_url,
                config.basic_auth,
                config.bear_auth_token,
                config.headers.unwrap_or_default(),
            )?,
        })
    }
}

#[async_trait]
impl<T> Map<Option<T>, Vec<u8>, ReqwestGetterConfig> for ReqwestGetter
where
    T: Render + Send + 'static,
{
    async fn map(&mut self, data: Option<T>) -> anyhow::Result<Vec<u8>> {
        self.get(data).await
    }
}

impl ReqwestGetter {
    async fn get<R>(&self, r: Option<R>) -> anyhow::Result<Vec<u8>>
    where
        R: Render,
    {
        let resp = self.client.get(r).await?;
        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }
}
