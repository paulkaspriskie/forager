//! Quote response type for FMP's `/quote` endpoint.

use serde::{Deserialize, Serialize};

/// A real-time quote for a single security.
///
/// Mirrors one element of the array returned by FMP's `/quote` endpoint.
/// The same struct is used in both directions: it is deserialized from
/// FMP's response, then serialized unchanged to API clients. Both sides use
/// camelCase (`changePercentage`, `priceAvg50`), so the frontend sees
/// exactly the field names FMP documents.
///
/// # Units
///
/// - Prices (`price`, `open`, `day_low`, ...) are in the security's trading
///   currency. FMP doesn't include the currency in this response.
/// - `change_percentage` is a **percentage, not a fraction**: `-1.9043`
///   means down 1.9%. Don't multiply it by 100 for display.
/// - `market_cap` is in whole currency units, not thousands or millions.
/// - `timestamp` is Unix time in **seconds** (not milliseconds), and marks
///   when the data was produced, not when it was fetched.
///
/// # Strictness
///
/// Every field is required and non-optional. If FMP omits a field or sends
/// `null` (which can happen for thinly traded or newly listed symbols),
/// deserialization of the whole quote fails and the API reports an
/// upstream error. Loosening this means changing the affected fields to
/// `Option<T>`, which the frontend would then have to handle.
///
/// Unknown fields are ignored, so FMP adding new fields won't break
/// parsing.
///
/// # Example
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Ticker symbol, e.g. `AAPL`.
    pub symbol: String,
    /// Company or instrument name.
    pub name: String,

    /// Latest traded price.
    pub price: f64,
    /// Absolute change from `previous_close`.
    pub change: f64,
    /// Change from `previous_close` as a percentage (`-1.9043` = -1.9%).
    pub change_percentage: f64,

    /// Shares traded so far in the current session.
    pub volume: u64,

    /// Lowest price of the current session.
    pub day_low: f64,
    /// Highest price of the current session.
    pub day_high: f64,
    /// Highest price over the past 52 weeks.
    pub year_high: f64,
    /// Lowest price over the past 52 weeks.
    pub year_low: f64,

    /// Market capitalization, in whole currency units.
    pub market_cap: u64,

    /// 50-day simple moving average of the closing price.
    ///
    /// Serialized as `priceAvg50`.
    pub price_avg50: f64,
    /// 200-day simple moving average of the closing price.
    ///
    /// Serialized as `priceAvg200`.
    pub price_avg200: f64,

    /// Exchange the security trades on, e.g. `NASDAQ`.
    pub exchange: String,

    /// Opening price of the current session.
    pub open: f64,
    /// Closing price of the previous session.
    pub previous_close: f64,

    /// When FMP produced this data, in Unix seconds.
    ///
    /// Outside market hours this can be hours old even for a freshly
    /// fetched quote, so display it rather than assuming the price is live.
    pub timestamp: i64,
}