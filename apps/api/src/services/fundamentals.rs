//! Fundamentals service: company financial ratios.
//!
//! Wraps the FMP client with a cache. Fundamentals only change when a
//! company files a report, so entries live for hours rather than seconds,
//! which keeps upstream calls (and FMP quota use) to a minimum.

use std::{sync::Arc, time::Duration};
use fmp_client::{FmpClient, FmpError, Ratios};
use moka::future::Cache;
use serde::Deserialize;
use crate::{error::AppError, symbol::Symbol};

/// Reporting period for fundamentals data.
///
/// Deserializes from lowercase query-string values (`annual`, `quarter`),
/// so it can be used directly in an axum `Query` extractor.
///
/// # Plan restrictions
///
/// [`Period::Quarter`] is not available on every FMP subscription. When the
/// current plan doesn't include it, FMP responds with `402 Payment
/// Required`, which surfaces here as [`AppError::PlanRestricted`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    /// Fiscal-year figures. FMP calls this `FY`.
    Annual,
    /// Quarterly figures.
    Quarter,
}

impl Period {
    /// Returns the value FMP expects in the `period` query parameter.
    ///
    /// This is deliberately separate from the `Deserialize` representation:
    /// the public API says `annual`, while FMP says `FY`. Keeping the
    /// mapping here means a change on either side touches one place.
    fn as_fmp(self) -> &'static str {
        match self {
            Period::Annual => "FY",
            Period::Quarter => "quarter",
        }
    }
}

/// Cache key for ratios lookups: `(symbol, period, limit)`.
///
/// All three parts are included so that, for example, a request for 3
/// annual records never collides with a request for 10, or with the
/// quarterly data for the same ticker.
type RatiosKey = (Symbol, Period, u32);

/// Fetches and caches fundamentals data from FMP.
///
/// Holds a cheap-to-clone [`FmpClient`] and an in-memory cache. It is
/// intended to be created once at startup and shared across requests behind
/// an [`Arc`].
///
/// # Caching behavior
///
/// - **TTL:** 6 hours, counted from insertion.
/// - **Capacity:** 2,000 entries, after which moka evicts the least
///   valuable ones.
/// - **Request coalescing:** concurrent requests for the same key trigger a
///   single upstream call; the others wait for its result.
/// - **Errors are not cached.** A transient failure (or a plan restriction)
///   is retried on the next request, which also means repeated requests for
///   a restricted resource each cost an upstream call.
pub struct FundamentalsService {
    fmp: FmpClient,
    ratios: Cache<RatiosKey, Arc<Vec<Ratios>>>,
}

impl FundamentalsService {
    /// Creates a new service backed by the given FMP client.
    ///
    /// The cache starts empty.
    pub fn new(fmp: FmpClient) -> Self {
        Self {
            fmp,
            ratios: Cache::builder()
                .max_capacity(2_000)
                .time_to_live(Duration::from_secs(6 * 60 * 60))
                .build(),
        }
    }

    /// Returns financial ratios for `symbol`, newest first.
    ///
    /// `limit` is the number of reporting periods to return. It is clamped
    /// to `1..=20` before use, so callers can't request unbounded history
    /// or fragment the cache with arbitrary values.
    ///
    /// The result is wrapped in an [`Arc`] so cache hits hand out a shared
    /// pointer instead of copying every ratio field.
    ///
    /// # Errors
    ///
    /// - [`AppError::NotFound`] if FMP has no ratios for the symbol.
    /// - [`AppError::PlanRestricted`] if the current FMP plan doesn't allow
    ///   the requested period or history length.
    /// - [`AppError::Upstream`] for any other failure (network error,
    ///   unexpected status, or unparseable response).
    pub async fn ratios(
        &self,
        symbol: &Symbol,
        period: Period,
        limit: u32,
    ) -> Result<Arc<Vec<Ratios>>, AppError> {
        let limit = limit.clamp(1, 20);
        let key = (symbol.clone(), period, limit);
        let fmp = self.fmp.clone();
        let sym = symbol.clone();

        self.ratios
            .try_get_with(key, async move {
                fmp.ratios(sym.as_str(), period.as_fmp(), limit).await.map(Arc::new)
            })
            .await
            .map_err(|e: Arc<FmpError>| e.into())
    }
}