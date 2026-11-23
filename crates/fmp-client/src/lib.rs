use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const BASE: &str = "https://financialmodelingprep.com/stable";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percentage: f64,
    pub volume: u64,
    pub day_low: f64,
    pub day_high: f64,
    pub year_high: f64,
    pub year_low: f64,
    pub market_cap: u64,
    pub price_avg50: f64,
    pub price_avg200: f64,
    pub exchange: String,
    pub open: f64,
    pub previous_close: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratios {
    pub symbol: String,
    pub date: String,
    pub fiscal_year: String,
    pub period: String,
    pub gross_profit_margin: Option<f64>,
    pub operating_profit_margin: Option<f64>,
    pub net_profit_margin: Option<f64>,
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub price_to_earnings_ratio: Option<f64>,
    pub price_to_book_ratio: Option<f64>,
    pub price_to_sales_ratio: Option<f64>,
    pub debt_to_equity_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
}

#[derive(Clone)]
pub struct FmpClient {
    http: Client,
    api_key: String,
}

impl FmpClient {
    pub fn new(api_key: String) -> Self {
        Self { http: Client::new(), api_key }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, String> {
        let resp = self
            .http
            .get(format!("{BASE}/{path}"))
            .query(query)
            .query(&[("apikey", &self.api_key)])
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?; // keeps the key out of logs

        if !resp.status().is_success() {
            return Err(format!("upstream status {}", resp.status()));
        }
        resp.json().await.map_err(|e| e.without_url().to_string())
    }

    pub async fn quote(&self, symbol: &str) -> Result<Option<Quote>, String> {
        let v: Vec<Quote> = self.get("quote", &[("symbol", symbol)]).await?;
        Ok(v.into_iter().next())
    }

    pub async fn ratios(&self, symbol: &str) -> Result<Vec<Ratios>, String> {
        self.get("ratios", &[("symbol", symbol), ("limit", "5")]).await
    }
}