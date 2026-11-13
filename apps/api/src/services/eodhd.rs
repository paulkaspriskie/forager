use reqwest::Client;
use serde_json::Value;


/// Client for communicating with the EODHD financial market data API.
///
/// Encapsulates HTTP communication and API authentication so that
/// other parts of the application do not need to manage API requests
/// or credentials directly.
#[derive(Clone)]
pub struct EodhdClient {
    /// Reusable HTTP client for making asynchronous API requests.
    client: Client,

    /// API token used to authenticate requests to EODHD.
    api_token: String,
}


impl EodhdClient {
    /// Creates a new EODHD API client.
    ///
    /// Accepts an API token, which is stored internally and included
    /// in subsequent requests to authenticate with EODHD.
    ///
    /// # Arguments
    ///
    /// * `api_token` - EODHD API authentication token.
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
        }
    }

    /// Retrieves historical end-of-day market data for a given symbol.
    ///
    /// Makes a request to the EODHD historical data endpoint and
    /// returns the response as a JSON value.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock ticker symbol, including exchange suffix (e.g. `AAPL.US`).
    /// * `from` - Beginning of the requested date range (YYYY-MM-DD).
    /// * `to` - End of the requested date range (YYYY-MM-DD).
    /// * `period` - Data interval (e.g. `d` for daily).
    ///
    /// # Returns
    ///
    /// Returns the historical market data as `serde_json::Value`.
    ///
    /// # Errors
    ///
    /// Returns a `reqwest::Error` if the HTTP request fails,
    /// the server returns an unsuccessful HTTP status, or
    /// the response cannot be deserialized into JSON.
    ///
    /// # API Endpoint
    ///
    /// `GET https://eodhd.com/api/eod/{symbol}`
    pub async fn get_historical_data(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
        period: &str,
    ) -> Result<Value, reqwest::Error> {
        let url = format!(
            "https://eodhd.com/api/eod/{}",
            symbol
        );

        // Build and execute the historical data request.
        let response = self.client
            .get(url)
            .query(&[
                ("api_token", self.api_token.as_str()),
                ("from", from),
                ("to", to),
                ("period", period),
                ("fmt", "json"),
            ])
            .send()
            .await?
            .error_for_status()?;

        // Deserialize the API response into a JSON value.
        response.json::<Value>().await
    }

    /// Retrieves fundamental company data for a given symbol.
    ///
    /// Makes a request to the EODHD fundamentals endpoint and
    /// returns company information, financial statements, valuation
    /// metrics, and other available fundamental data as JSON.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock ticker symbol, including exchange suffix (e.g. `AAPL.US`).
    ///
    /// # Returns
    ///
    /// Returns the company fundamentals as `serde_json::Value`.
    ///
    /// # Errors
    ///
    /// Returns a `reqwest::Error` if the HTTP request fails,
    /// the server returns an unsuccessful HTTP status, or
    /// the response cannot be deserialized into JSON.
    ///
    /// # API Endpoint
    ///
    /// `GET https://eodhd.com/api/v1.1/fundamentals/{symbol}`
    pub async fn get_fundamentals(
        &self,
        symbol: &str,
    ) -> Result<Value, reqwest::Error> {
        let url = format!(
            "https://eodhd.com/api/v1.1/fundamentals/{}",
            symbol
        );

        // Build and execute the fundamentals request.
        let response = self.client
            .get(url)
            .query(&[
                ("api_token", self.api_token.as_str()),
                ("fmt", "json"),
            ])
            .send()
            .await?
            .error_for_status()?;

        // Deserialize the API response into a JSON value.
        response.json::<Value>().await
    }
}
