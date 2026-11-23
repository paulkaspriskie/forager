//! Market service: real-time stock quotes.
//!
//! Wraps the FMP client with a short-lived cache. Quotes change constantly,
//! so entries expire in seconds. That is long enough to absorb bursts of
//! identical requests (several browser tabs, or a frontend polling on an
//! interval) but short enough that the data still feels live.

use std::{sync::Arc, time::Duration};
use fmp_client::{FmpClient, FmpError, Quote};
use moka::future::Cache;
use crate::{error::AppError, symbol::Symbol};

/// Fetches and caches market data from FMP.
///
/// Holds a cheap-to-clone [`FmpClient`] and an in-memory cache. It is
/// intended to be created once at startup and shared across requests behind
/// an [`Arc`].
///
/// # Caching behavior
///
/// - **TTL:** 10 seconds, counted from insertion. Every request within that
///   window for the same symbol is served from memory.
/// - **Capacity:** 5,000 symbols, after which moka evicts the least
///   valuable entries.
/// - **Request coalescing:** concurrent requests for the same symbol
///   trigger a single upstream call; the others wait for its result.
/// - **Errors are not cached.** A failed lookup is retried on the next
///   request, so repeated requests for an unknown or restricted symbol
///   each cost an upstream call.
///
/// With the 10-second TTL, a single symbol costs at most 6 upstream calls
/// per minute no matter how many clients are watching it.
pub struct MarketService {
    fmp: FmpClient,
    quotes: Cache<Symbol, Quote>,
}

impl MarketService {
    /// Creates a new service backed by the given FMP client.
    ///
    /// The cache starts empty.
    pub fn new(fmp: FmpClient) -> Self {
        Self {
            fmp,
            quotes: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(10))
                .build(),
        }
    }

    /// Returns the latest quote for `symbol`.
    ///
    /// On a cache hit this returns a clone of the stored [`Quote`]. On a
    /// miss it calls FMP, stores the result, and returns it. The `Symbol`
    /// type guarantees the input is already validated and uppercased, so
    /// `aapl` and `AAPL` share one cache entry.
    ///
    /// The `timestamp` field on the returned quote is FMP's data time (Unix
    /// seconds), not the time it was cached. Use it to show how fresh the
    /// price is, which matters outside market hours.
    ///
    /// # Errors
    ///
    /// - [`AppError::NotFound`] if FMP has no quote for the symbol.
    /// - [`AppError::PlanRestricted`] if the symbol or exchange isn't
    ///   covered by the current FMP plan.
    /// - [`AppError::Upstream`] for any other failure (network error,
    ///   unexpected status, or unparseable response).
    pub async fn quote(&self, symbol: &Symbol) -> Result<Quote, AppError> {
        let fmp = self.fmp.clone();
        let sym = symbol.clone();

        self.quotes
            .try_get_with(symbol.clone(), async move { fmp.quote(sym.as_str()).await })
            .await
            .map_err(|e: Arc<FmpError>| e.into())
    }
}