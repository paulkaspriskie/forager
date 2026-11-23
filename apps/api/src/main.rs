mod error;
mod routes;
mod services;
mod state;
mod symbol;

use std::sync::Arc;
use fmp_client::FmpClient;
use services::{fundamentals::FundamentalsService, market::MarketService};
use state::AppState;
use tower_http::trace::TraceLayer;

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