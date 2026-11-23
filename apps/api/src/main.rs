//! Financial dashboard API server.
//!
//! A thin HTTP layer over the [Financial Modeling Prep][fmp] (FMP) API. It
//! keeps the FMP API key server-side, validates input, and caches upstream
//! responses so the frontend can refetch freely without burning through the
//! FMP rate limit.
//!
//! # Endpoints
//!
//! | Route                                        | Data                         |
//! |----------------------------------------------|------------------------------|
//! | `GET /api/health`                            | Liveness check               |
//! | `GET /api/market/quote/:symbol`              | Real-time quote              |
//! | `GET /api/fundamentals/:symbol/ratios`       | Financial ratios (`period`, `limit`) |
//!
//! # Architecture
//!
//! Requests flow through three layers, each with a single responsibility:
//!
//! 1. **[`routes`]**: parse and validate the request, map errors to HTTP
//!    responses. No business logic.
//! 2. **[`services`]**: own caching and orchestration. Call the FMP client
//!    only on a cache miss.
//! 3. **`fmp_client`**: typed HTTP client for FMP. The only code that knows
//!    about the API key and upstream URLs.
//!
//! # Configuration
//!
//! | Variable      | Required | Description                                  |
//! |---------------|----------|----------------------------------------------|
//! | `FMP_API_KEY` | yes      | FMP API key. Loaded from `.env` if present.  |
//!
//! [fmp]: https://site.financialmodelingprep.com/developer/docs

/// Application error type and its mapping to HTTP responses.
///
/// Defines [`AppError`](error::AppError), which converts upstream failures
/// into client-safe status codes and generic messages while the detailed
/// cause goes to the server logs.
mod error;

/// HTTP route definitions.
///
/// Thin axum handlers that validate input, delegate to a service, and
/// return JSON. All routes are nested under `/api`.
mod routes;

/// Business logic and caching.
///
/// Each service wraps the FMP client with a cache whose TTL matches how
/// quickly that data changes (seconds for quotes, hours for ratios).
mod services;

/// Shared application state.
///
/// [`AppState`] holds the services and is cloned into every request
/// handler. Cloning is cheap because the services sit behind [`Arc`].
mod state;

/// Validated ticker symbol type.
///
/// [`Symbol`](symbol::Symbol) normalizes and validates user-supplied
/// tickers so malformed input never reaches a cache key or an upstream
/// request.
mod symbol;

use std::sync::Arc;
use fmp_client::FmpClient;
use services::{fundamentals::FundamentalsService, market::MarketService};
use state::AppState;
use tower_http::trace::TraceLayer;

/// Starts the API server.
///
/// Startup sequence:
///
/// 1. Initialize `tracing` for structured logging.
/// 2. Load environment variables from a `.env` file, if one exists. A
///    missing file is logged as a warning, not an error, so the server
///    also works with variables set directly in the environment.
/// 3. Read `FMP_API_KEY` and build a single shared [`FmpClient`].
/// 4. Construct the services, each holding its own cache, and bundle them
///    into [`AppState`].
/// 5. Build the router, attach request tracing, and serve on
///    `127.0.0.1:3001`.
///
/// The server binds to localhost only. The frontend is expected to reach it
/// through a dev-server proxy (e.g. Vite's `server.proxy`) rather than
/// directly from the browser.
///
/// # Errors
///
/// Returns an error if:
///
/// - `FMP_API_KEY` is not set in the environment or `.env`.
/// - The TCP listener cannot bind to the address (for example, the port is
///   already in use).
/// - The server fails while running.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    match dotenvy::dotenv() {
        Ok(path) => tracing::info!("loaded env from {}", path.display()),
        Err(e) => tracing::warn!("no .env loaded: {e}"),
    }

    let api_key = std::env::var("FMP_API_KEY")
        .map_err(|_| anyhow::anyhow!("FMP_API_KEY must be set"))?;
    let fmp = FmpClient::new(api_key);

    let state = AppState {
        market: Arc::new(MarketService::new(fmp.clone())),
        fundamentals: Arc::new(FundamentalsService::new(fmp)),
    };

    let app = routes::api().with_state(state).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    tracing::info!("listening on http://127.0.0.1:3001");
    axum::serve(listener, app).await?;
    Ok(())
}