use pipebase::common::Render;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, Response,
};

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

pub struct ReqwestClient {
    client: Client,
    url: String,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
    headers: HeaderMap,
}

impl ReqwestClient {
    pub fn new(
        url: String,
        basic_auth: Option<BasicAuth>,
        bearer_auth_token: Option<String>,
        headers: HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let mut hmap = HeaderMap::new();
        for (name, value) in &headers {
            hmap.insert::<HeaderName>(name.parse()?, value.parse()?);
        }
        Ok(ReqwestClient {
            client: Client::new(),
            url,
            basic_auth,
            bearer_auth_token,
            headers: hmap,
        })
    }

    pub async fn post<B>(&self, body: B) -> anyhow::Result<Response>
    where
        B: Into<Body>,
    {
        let req = self.client.post(&self.url).headers(self.headers.to_owned());
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

    pub async fn query<Q>(&self, query: Option<Q>) -> anyhow::Result<Response>
    where
        Q: Serialize,
    {
        let req = self.client.get(&self.url).headers(self.headers.to_owned());
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

    pub async fn get<R>(&self, r: Option<R>) -> anyhow::Result<Response>
    where
        R: Render,
    {
        let url = match r {
            Some(r) => {
                let path = r.render();
                let url = match path.starts_with('/') {
                    true => format!("{}{}", self.url, path),
                    false => format!("{}/{}", self.url, path),
                };
                url
            }
            None => self.url.to_owned(),
        };
        let req = self.client.get(&url).headers(self.headers.to_owned());
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
}
