//! HTTP routes for fundamentals data.
//!
//! Exposes company financial ratios under `/api/fundamentals`. The handler
//! is deliberately thin: it validates the ticker, applies defaults for
//! optional query parameters, and delegates to
//! [`FundamentalsService`](crate::services::fundamentals::FundamentalsService).
//! Caching and upstream calls happen there, not here.

use std::sync::Arc;
use axum::{extract::{Path, Query, State}, routing::get, Json, Router};
use fmp_client::Ratios;
use serde::Deserialize;
use crate::{error::AppError, services::fundamentals::Period, state::AppState, symbol::Symbol};

/// Query-string parameters for the ratios endpoint.
///
/// Both fields are optional; defaults are applied in the handler.
#[derive(Deserialize)]
struct RatiosQuery {
    /// Reporting period: `annual` or `quarter`. Defaults to `annual`.
    period: Option<Period>,
    /// Number of reporting periods to return. Defaults to `5`. The service
    /// clamps this to `1..=20`, so out-of-range values are adjusted rather
    /// than rejected.
    limit: Option<u32>,
}

/// Builds the router for fundamentals routes.
///
/// Paths here are relative: the parent router nests this under `/api`, so
/// the full path of the route below is
/// `/api/fundamentals/:symbol/ratios`.
///
/// | Method | Path                                | Handler      |
/// |--------|-------------------------------------|--------------|
/// | `GET`  | `/fundamentals/:symbol/ratios`      | [`get_ratios`] |
pub fn router() -> Router<AppState> {
    Router::new().route("/fundamentals/:symbol/ratios", get(get_ratios))
}

/// `GET /api/fundamentals/:symbol/ratios`
///
/// Returns financial ratios for a company as a JSON array, newest period
/// first. Each element covers one reporting period and includes
/// profitability, liquidity, valuation, leverage, and per-share metrics.
///
/// # Path parameters
///
/// - `symbol`: ticker such as `AAPL`. Case-insensitive; normalized to
///   uppercase and validated before use.
///
/// # Query parameters
///
/// - `period`: `annual` (default) or `quarter`.
/// - `limit`: number of periods, default `5`, clamped to `1..=20`.
///
/// # Example
///
/// ```text
/// GET /api/fundamentals/aapl/ratios?period=annual&limit=3
/// ```
///
/// # Responses
///
/// | Status | When                                                      |
/// |--------|-----------------------------------------------------------|
/// | `200`  | Ratios found                                              |
/// | `400`  | Invalid symbol, or malformed query string (see below)     |
/// | `402`  | Requested data isn't included in the current FMP plan     |
/// | `404`  | FMP has no ratios for the symbol                          |
/// | `502`  | Upstream failure (network error, bad response)            |
///
/// Errors from [`AppError`] are returned as `{"error": "<message>"}`.
///
/// # Caveats
///
/// - **Query-string rejections aren't JSON.** An unknown `period` (e.g.
///   `?period=monthly`) or a non-numeric `limit` is rejected by axum's
///   `Query` extractor before this handler runs, so that `400` carries
///   axum's plain-text message rather than the `{"error": ...}` shape.
/// - **`period=quarter` may return `402`** on FMP plans that don't include
///   quarterly data. The frontend should treat this as "feature locked"
///   rather than an outage.
async fn get_ratios(
    State(s): State<AppState>,
    Path(raw): Path<String>,
    Query(q): Query<RatiosQuery>,
) -> Result<Json<Arc<Vec<Ratios>>>, AppError> {
    let symbol = Symbol::parse(&raw).ok_or(AppError::BadSymbol)?;
    let data = s
        .fundamentals
        .ratios(&symbol, q.period.unwrap_or(Period::Annual), q.limit.unwrap_or(5))
        .await?;
    Ok(Json(data))
}