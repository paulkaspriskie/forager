use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::get,
  Json, Router,
};
use fmp_client::{FmpClient, Quote, Ratios};

async fn quote(
  State(fmp): State<FmpClient>,
  Path(symbol): Path<String>,
) -> Result<Json<Quote>, StatusCode> {
  match fmp.quote(&symbol).await {
      Ok(Some(q)) => Ok(Json(q)),
      Ok(None) => Err(StatusCode::NOT_FOUND),
      Err(e) => {
          eprintln!("fmp quote error: {e}");
          Err(StatusCode::BAD_GATEWAY)
      }
  }
}

async fn ratios(
  State(fmp): State<FmpClient>,
  Path(symbol): Path<String>,
) -> Result<Json<Vec<Ratios>>, StatusCode> {
  fmp.ratios(&symbol).await.map(Json).map_err(|e| {
      eprintln!("fmp ratios error: {e}");
      StatusCode::BAD_GATEWAY
  })
}

#[tokio::main]
async fn main() {
  dotenvy::dotenv().ok();
  let key = std::env::var("FMP_API_KEY").expect("FMP_API_KEY must be set");

  let app = Router::new()
      .route("/api/market/quote/:symbol", get(quote))
      .route("/api/fundamentals/:symbol/ratios", get(ratios))
      .with_state(FmpClient::new(key));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
  println!("listening on http://127.0.0.1:3001");
  axum::serve(listener, app).await.unwrap();
}