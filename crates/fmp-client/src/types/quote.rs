use serde::{Deserialize, Serialize};

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