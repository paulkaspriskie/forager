//! Shared application state.
//!
//! Defines [`AppState`], the single value axum clones into every request
//! handler. It is built once in `main` and attached to the router with
//! `with_state`.

use std::sync::Arc;
use crate::services::{fundamentals::FundamentalsService, market::MarketService};

/// State shared by all request handlers.
///
/// Each field is a service that owns its own cache and a handle to the FMP
/// client. Handlers reach them with the `State` extractor:
///
/// ```ignore
/// async fn get_quote(State(s): State<AppState>, /* ... */) {
///     s.market.quote(&symbol).await
/// }
/// ```
///
/// # Cloning
///
/// Axum clones the state for every request, so `Clone` has to be cheap.
/// Each service sits behind an [`Arc`], which means a clone copies two
/// pointers and nothing else. Every request therefore sees the same
/// service instances, and the same caches, rather than private copies.
///
/// # Concurrency
///
/// No locks are needed here. The services are immutable after construction,
/// and their caches (moka) handle concurrent access internally.
///
/// # Adding a service
///
/// To expose a new data source:
///
/// 1. Create the service in `services/` and add it to `services/mod.rs`.
/// 2. Add an `Arc<NewService>` field below.
/// 3. Construct it in `main`, passing `fmp.clone()` if it needs the client.
#[derive(Clone)]
pub struct AppState {
    /// Real-time quotes. Short-lived cache (10 seconds).
    pub market: Arc<MarketService>,
    /// Company financial ratios. Long-lived cache (6 hours).
    pub fundamentals: Arc<FundamentalsService>,
}