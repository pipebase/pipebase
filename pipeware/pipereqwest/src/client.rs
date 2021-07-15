use serde::Deserialize;
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

pub struct ReqwestPostClient {
    client: Client,
    url: String,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
    headers: HeaderMap,
}

impl ReqwestPostClient {
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
        Ok(ReqwestPostClient {
            client: Client::new(),
            url: url,
            basic_auth: basic_auth,
            bearer_auth_token: bearer_auth_token,
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
}