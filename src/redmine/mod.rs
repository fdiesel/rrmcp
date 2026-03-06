pub mod common;
pub mod issues;
pub mod projects;

use reqwest::{Client, Method, RequestBuilder};

use crate::error::RedmineError;

#[derive(Clone, Debug)]
pub struct RedmineClient {
    base_url: String,
    http: Client,
}

impl RedmineClient {
    pub fn new(base_url: String) -> anyhow::Result<Self> {
        let http = Client::builder().build()?;
        Ok(Self { base_url, http })
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub async fn read<T>(&self, path: &str, api_key: &str) -> Result<T, RedmineError>
    where
        T: serde::de::DeserializeOwned,
    {
        let res = self
            .http
            .request(Method::GET, self.url(path))
            .header("X-Redmine-API-KEY", api_key)
            .send()
            .await?;
        let text = Self::check_response(res).await?;
        let parsed: T = serde_json::from_str(&text)
            .map_err(|e| RedmineError::UnexpectedResponse(e.to_string()))?;
        Ok(parsed)
    }

    pub fn get(&self, path: &str, api_key: &str) -> RequestBuilder {
        self.http
            .request(Method::GET, self.url(path))
            .header("X-Redmine-API-Key", api_key)
    }

    pub fn post(&self, path: &str, api_key: &str) -> RequestBuilder {
        self.http
            .request(Method::POST, self.url(path))
            .header("X-Redmine-API-Key", api_key)
            .header("Content-Type", "application/json")
    }

    pub fn put(&self, path: &str, api_key: &str) -> RequestBuilder {
        self.http
            .request(Method::PUT, self.url(path))
            .header("X-Redmine-API-Key", api_key)
            .header("Content-Type", "application/json")
    }

    pub fn delete(&self, path: &str, api_key: &str) -> RequestBuilder {
        self.http
            .request(Method::DELETE, self.url(path))
            .header("X-Redmine-API-Key", api_key)
    }

    /// Check a response for errors and return the body text.
    pub async fn check_response(resp: reqwest::Response) -> Result<String, RedmineError> {
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        if status >= 200 && status < 300 {
            Ok(body)
        } else {
            Err(RedmineError::from_status(status, &body))
        }
    }
}
