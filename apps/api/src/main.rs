mod services;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

use services::eodhd::EodhdClient;


/// Query parameters accepted by the historical market data endpoint.
///
/// The `from` and `to` fields define the requested date range.
/// The `period` field determines the data interval and defaults to daily.
#[derive(Deserialize)]
struct HistoryQuery {
    from: String,
    to: String,

    /// Defaults to daily data when no period is provided.
    #[serde(default = "default_period")]
    period: String,
}

/// Returns the default interval for historical market data.
///
/// EODHD uses `d` to represent daily intervals.
fn default_period() -> String {
    "d".to_string()
}


/// Validates a stock ticker symbol before passing it to EODHD.
///
/// Allows alphanumeric characters and common symbol characters:
/// periods, hyphens, underscores, and carets.
///
/// This provides basic input validation and prevents unexpected
/// characters from being included in upstream API requests.
fn valid_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 30
        && symbol.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '.' | '-' | '_' | '^')
        })
}


/// Health check endpoint.
///
/// Used to verify that the API server is running and responding
/// to HTTP requests.
///
/// GET /api/health
async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "application": "Forager API"
    }))
}


/// Retrieves historical end-of-day market data for a given symbol.
///
/// Accepts a ticker symbol as a path parameter and a date range
/// with an optional data interval as query parameters.
///
/// GET /api/market/history/{symbol}?from=YYYY-MM-DD&to=YYYY-MM-DD
///
/// Returns the historical market data provided by EODHD.
/// Invalid symbols return HTTP 400.
/// Upstream API failures return HTTP 502.
async fn history(
    State(client): State<EodhdClient>,
    Path(symbol): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate the ticker symbol before making an external request.
    if !valid_symbol(&symbol) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid ticker symbol".to_string(),
        ));
    }

    // Delegate the external API request to the EODHD service.
    let data = client
        .get_historical_data(
            &symbol,
            &params.from,
            &params.to,
            &params.period,
        )
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_GATEWAY,
                format!("EODHD historical request failed: {err}"),
            )
        })?;

    // Return the upstream response as JSON.
    Ok(Json(data))
}


/// Retrieves fundamental company data for a given ticker symbol.
///
/// Accepts a ticker symbol as a path parameter and delegates
/// the request to the EODHD fundamentals endpoint.
///
/// GET /api/market/fundamentals/{symbol}
///
/// Returns the company fundamentals provided by EODHD.
/// Invalid symbols return HTTP 400.
/// Upstream API failures return HTTP 502.
async fn fundamentals(
    State(client): State<EodhdClient>,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate the ticker symbol before making an external request.
    if !valid_symbol(&symbol) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid ticker symbol".to_string(),
        ));
    }

    // Delegate the external API request to the EODHD service.
    let data = client
        .get_fundamentals(&symbol)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_GATEWAY,
                format!("EODHD fundamentals request failed: {err}"),
            )
        })?;

    // Return the upstream response as JSON.
    Ok(Json(data))
}


/// Application entry point.
///
/// Initializes environment configuration, creates the EODHD client,
/// registers API routes, and starts the Axum HTTP server.
///
/// The EODHD API token is loaded from the EODHD_API_TOKEN environment
/// variable, typically supplied through the project's `.env` file.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from the .env file when available.
    dotenvy::dotenv().ok();

    // Retrieve the EODHD API token from the environment.
    let api_token = env::var("EODHD_API_TOKEN")?;

    // Initialize the shared EODHD client.
    let eodhd = EodhdClient::new(api_token);

    // Register application routes and provide shared application state.
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/market/history/{symbol}", get(history))
        .route("/api/market/fundamentals/{symbol}", get(fundamentals))
        .with_state(eodhd);

    // Bind the HTTP server to localhost on port 3000.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await?;

    println!("Forager API running at http://localhost:3000");

    // Start serving incoming HTTP requests.
    axum::serve(listener, app).await?;

    Ok(())
}
