//! Typed HTTP client for the Financial Modeling Prep (FMP) `stable` API.
//!
//! This is the only code in the workspace that knows about FMP's base URL
//! and API key. Callers get typed results ([`Quote`], [`Ratios`]) and a
//! small error enum, and never touch raw URLs or JSON.
//!
//! The client does no caching and no retries. Each call is one HTTP
//! request, so caching belongs in the layer above (the API's services).

use std::time::Duration;
use reqwest::Client;
use serde::de::DeserializeOwned;
use crate::types::{Quote, Ratios};

/// Base URL for FMP's `stable` endpoints. Paths are appended to this.
const BASE: &str = "https://financialmodelingprep.com/stable";

/// Errors returned by [`FmpClient`].
///
/// Variants carry details for server-side logging. They are meant to be
/// mapped to a coarse, client-safe error by the caller, not shown to end
/// users: [`Status`](FmpError::Status) and [`Http`](FmpError::Http) can
/// reveal information about the data provider.
#[derive(Debug, thiserror::Error)]
pub enum FmpError {
    /// The request failed before a usable response was received, or the
    /// response body could not be read or decoded.
    ///
    /// This covers connection failures, timeouts (10 seconds), and JSON
    /// that doesn't match the expected type. The URL is stripped from the
    /// inner error so the API key can't leak through logs. That also means
    /// the message won't say *which* request failed, so log the endpoint
    /// at the call site if you need that.
    #[error("request failed: {0}")]
    Http(reqwest::Error),

    /// FMP responded with a non-2xx status.
    ///
    /// Carries only the numeric status code. Notable cases:
    ///
    /// - `401`/`403`: invalid or revoked API key.
    /// - `402`: the request isn't available on your FMP plan (for example,
    ///   `period=quarter` on a plan without quarterly data).
    /// - `429`: rate limit exceeded.
    ///
    /// The response body, which often explains the cause, is not captured.
    #[error("upstream returned status {0}")]
    Status(u16),

    /// FMP returned a successful response containing no data.
    ///
    /// FMP signals an unknown ticker with an empty array rather than an
    /// error status, so this is how "no such symbol" shows up.
    #[error("symbol not found")]
    NotFound,
}

/// Client for FMP's `stable` API.
///
/// Holds a connection-pooled [`reqwest::Client`] and the API key. It is
/// cheap to clone (reqwest's client is an `Arc` internally), so create one
/// at startup and clone it into each service rather than building
/// several.
///
/// `Debug` is intentionally not derived, so the API key can't end up in a
/// log line through `{:?}`.
#[derive(Clone)]
pub struct FmpClient {
    http: Client,
    api_key: String,
}

impl FmpClient {
    /// Creates a client that authenticates with `api_key`.
    ///
    /// Requests time out after 10 seconds. The key is sent as the `apikey`
    /// query parameter on every request, which is how FMP expects it.
    ///
    /// # Panics
    ///
    /// Panics if the underlying HTTP client can't be built, which in
    /// practice means a broken TLS setup. This happens at startup, so
    /// failing fast is intentional.
    pub fn new(api_key: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build http client");
        Self { http, api_key }
    }

    /// Sends a `GET` to `{BASE}/{path}` and deserializes the JSON body.
    ///
    /// `query` holds the endpoint's parameters. The API key is appended
    /// separately, so callers never handle it. Values are URL-encoded by
    /// reqwest, so a hostile string can't inject extra parameters.
    ///
    /// # Errors
    ///
    /// - [`FmpError::Http`] on transport failure, timeout, or a body that
    ///   doesn't deserialize into `T`.
    /// - [`FmpError::Status`] on any non-2xx response.
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

    /// Fetches the latest quote for `symbol` from `/quote`.
    ///
    /// FMP returns an array even for a single symbol; this unwraps it to
    /// the first element. `symbol` is passed through as given, so callers
    /// should validate and normalize it first (the API crate does this
    /// with its `Symbol` type).
    ///
    /// # Errors
    ///
    /// - [`FmpError::NotFound`] if FMP returns an empty array.
    /// - Anything [`get`](Self::get) can return.
    pub async fn quote(&self, symbol: &str) -> Result<Quote, FmpError> {
        let v: Vec<Quote> = self.get("quote", &[("symbol", symbol)]).await?;
        v.into_iter().next().ok_or(FmpError::NotFound)
    }

    /// Fetches financial ratios for `symbol` from `/ratios`, newest first.
    ///
    /// - `period` is passed to FMP as-is. This client doesn't know the
    ///   valid values; the API crate sends `FY` for annual and `quarter`
    ///   for quarterly. Quarterly may be rejected with a `402` depending
    ///   on your FMP plan.
    /// - `limit` is the number of reporting periods to request. It is not
    ///   bounded here, so callers should clamp it. Plans may also cap how
    ///   much history is returned.
    ///
    /// # Errors
    ///
    /// - [`FmpError::NotFound`] if FMP returns an empty array.
    /// - Anything [`get`](Self::get) can return.
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