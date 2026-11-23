//! HTTP routes for market data.
//!
//! Exposes real-time quotes under `/api/market`. The handler is
//! deliberately thin: it validates the ticker and delegates to
//! [`MarketService`](crate::services::market::MarketService). Caching and
//! upstream calls happen there, not here.

use axum::{extract::{Path, State}, routing::get, Json, Router};
use fmp_client::Quote;
use crate::{error::AppError, state::AppState, symbol::Symbol};

/// Builds the router for market data routes.
///
/// Paths here are relative: the parent router nests this under `/api`, so
/// the full path of the route below is `/api/market/quote/:symbol`.
///
/// | Method | Path                      | Handler       |
/// |--------|---------------------------|---------------|
/// | `GET`  | `/market/quote/:symbol`   | [`get_quote`] |
pub fn router() -> Router<AppState> {
    Router::new().route("/market/quote/:symbol", get(get_quote))
}

/// `GET /api/market/quote/:symbol`
///
/// Returns the latest quote for a single ticker as a JSON object (not an
/// array, even though FMP wraps its response in one). Includes price, daily
/// change, volume, the day and 52-week ranges, market cap, and the 50 and
/// 200-day moving averages.
///
/// # Path parameters
///
/// - `symbol`: ticker such as `AAPL`. Case-insensitive; normalized to
///   uppercase and validated before use.
///
/// # Example
///
/// ```text
/// GET /api/market/quote/aapl
/// ```
///
/// ```json
/// {
///   "symbol": "AAPL",
///   "name": "Apple Inc.",
///   "price": 342.56,
///   "change": -6.65,
///   "changePercentage": -1.9043,
///   "volume": 80692545,
///   "dayLow": 340.56,
///   "dayHigh": 354.3,
///   "yearHigh": 360.83,
///   "yearLow": 243.42,
///   "marketCap": 4978150432000,
///   "priceAvg50": 341.8072,
///   "priceAvg200": 303.2406,
///   "exchange": "NASDAQ",
///   "open": 352.63,
///   "previousClose": 349.21,
///   "timestamp": 1795208402
/// }
/// ```
///
/// # Responses
///
/// | Status | When                                                   |
/// |--------|--------------------------------------------------------|
/// | `200`  | Quote found                                            |
/// | `400`  | Invalid symbol                                         |
/// | `402`  | Symbol or exchange isn't included in the FMP plan     |
/// | `404`  | FMP has no quote for the symbol                        |
/// | `502`  | Upstream failure (network error, bad response)         |
///
/// Errors from [`AppError`] are returned as `{"error": "<message>"}`.
///
/// # Caveats
///
/// - **Quotes are cached for 10 seconds**, so two requests inside that
///   window return identical data. Poll no faster than that.
/// - **`timestamp` is the time of the data (Unix seconds)**, not the time
///   of the request. Outside market hours it will be hours old, so show it
///   in the UI rather than presenting the price as live.
async fn get_quote(
    State(s): State<AppState>,
    Path(raw): Path<String>,
) -> Result<Json<Quote>, AppError> {
    let symbol = Symbol::parse(&raw).ok_or(AppError::BadSymbol)?;
    Ok(Json(s.market.quote(&symbol).await?))
}