use async_trait::async_trait;
use pipebase::common::{ConfigInto, FromConfig, FromPath};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, IntoUrl, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}

impl BasicAuth {
    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }
}

#[derive(Deserialize)]
pub struct HttpClientConfig {
    basic_auth: Option<BasicAuth>,
    bear_auth_token: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl FromPath for HttpClientConfig {}

pub struct HttpClient {
    client: Client,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
    headers: HeaderMap,
}

impl HttpClient {
    fn build_header_map(headers: &HashMap<String, String>) -> anyhow::Result<HeaderMap> {
        let mut hmap = HeaderMap::new();
        for (name, value) in headers {
            hmap.insert::<HeaderName>(name.parse()?, value.parse()?);
        }
        Ok(hmap)
    }

    pub fn new(
        basic_auth: Option<BasicAuth>,
        bearer_auth_token: Option<String>,
        headers: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Self> {
        let headers = match headers {
            Some(headers) => Self::build_header_map(&headers)?,
            None => HeaderMap::new(),
        };
        Ok(HttpClient {
            client: Client::new(),
            basic_auth,
            bearer_auth_token,
            headers,
        })
    }

    pub async fn post<U, B>(&self, url: U, body: B) -> anyhow::Result<Response>
    where
        U: IntoUrl,
        B: Into<Body>,
    {
        let req = self.client.post(url).headers(self.headers.to_owned());
        let req = match self.basic_auth {
            Some(ref basic_auth) => req.basic_auth(basic_auth.username(), basic_auth.password()),
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.body(body).send().await?;
        Ok(resp)
    }

    pub async fn query<U, Q>(&self, url: U, query: Option<Q>) -> anyhow::Result<Response>
    where
        U: IntoUrl,
        Q: Serialize,
    {
        let req = self.client.get(url).headers(self.headers.to_owned());
        let req = match query {
            Some(ref query) => req.query(query),
            None => req,
        };
        let req = match self.basic_auth {
            Some(ref basic_auth) => req.basic_auth(basic_auth.username(), basic_auth.password()),
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.send().await?;
        Ok(resp)
    }

    pub async fn query_json<U, R, Q>(&self, url: U, query: Option<Q>) -> anyhow::Result<R>
    where
        U: IntoUrl,
        R: DeserializeOwned,
        Q: Serialize,
    {
        let response = self.query(url, query).await?;
        let bytes = response.bytes().await?;
        let bytes = bytes.to_vec();
        let body = serde_json::from_slice::<R>(&bytes)?;
        Ok(body)
    }

    pub async fn get<U>(&self, url: U) -> anyhow::Result<Response>
    where
        U: IntoUrl,
    {
        let req = self.client.get(url).headers(self.headers.to_owned());
        let req = match self.basic_auth {
            Some(ref basic_auth) => req.basic_auth(basic_auth.username(), basic_auth.password()),
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.send().await?;
        Ok(resp)
    }

    pub async fn get_json<U, R>(&self, url: U) -> anyhow::Result<R>
    where
        U: IntoUrl,
        R: DeserializeOwned,
    {
        let response = self.get(url).await?;
        let bytes = response.bytes().await?;
        let bytes = bytes.to_vec();
        let body = serde_json::from_slice::<R>(&bytes)?;
        Ok(body)
    }
}

impl ConfigInto<HttpClient> for HttpClientConfig {}

#[async_trait]
impl FromConfig<HttpClientConfig> for HttpClient {
    async fn from_config(config: HttpClientConfig) -> anyhow::Result<Self> {
        Self::new(config.basic_auth, config.bear_auth_token, config.headers)
    }
}
