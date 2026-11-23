// src/client.rs
use std::time::Duration;
use reqwest::Client;
use serde::de::DeserializeOwned;
use crate::types::{Quote, Ratios};

const BASE: &str = "https://financialmodelingprep.com/stable";

#[derive(Debug, thiserror::Error)]
pub enum FmpError {
    #[error("request failed: {0}")]
    Http(reqwest::Error),
    #[error("upstream returned status {0}")]
    Status(u16),
    #[error("symbol not found")]
    NotFound,
}

#[derive(Clone)]
pub struct FmpClient {
    http: Client,
    api_key: String,
}

impl FmpClient {
    pub fn new(api_key: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build http client");
        Self { http, api_key }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, FmpError> {
        let resp = self
            .http
            .get(format!("{BASE}/{path}"))
            .query(query)
            .query(&[("apikey", &self.api_key)])
            .send()
            .await
            .map_err(|e| FmpError::Http(e.without_url()))?; // keeps the key out of logs

        if !resp.status().is_success() {
            return Err(FmpError::Status(resp.status().as_u16()));
        }
        resp.json().await.map_err(|e| FmpError::Http(e.without_url()))
    }

    pub async fn quote(&self, symbol: &str) -> Result<Quote, FmpError> {
        let v: Vec<Quote> = self.get("quote", &[("symbol", symbol)]).await?;
        v.into_iter().next().ok_or(FmpError::NotFound)
    }

    pub async fn ratios(
        &self,
        symbol: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<Ratios>, FmpError> {
        let v: Vec<Ratios> = self
            .get(
                "ratios",
                &[("symbol", symbol), ("period", period), ("limit", &limit.to_string())],
            )
            .await?;
        if v.is_empty() { Err(FmpError::NotFound) } else { Ok(v) }
    }
}