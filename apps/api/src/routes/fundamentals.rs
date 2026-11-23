use std::sync::Arc;
use axum::{extract::{Path, Query, State}, routing::get, Json, Router};
use fmp_client::Ratios;
use serde::Deserialize;
use crate::{error::AppError, services::fundamentals::Period, state::AppState, symbol::Symbol};

#[derive(Deserialize)]
struct RatiosQuery {
    period: Option<Period>,
    limit: Option<u32>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/fundamentals/:symbol/ratios", get(get_ratios))
}

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