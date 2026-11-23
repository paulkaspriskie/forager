use axum::{extract::{Path, State}, routing::get, Json, Router};
use fmp_client::Quote;
use crate::{error::AppError, state::AppState, symbol::Symbol};

pub fn router() -> Router<AppState> {
    Router::new().route("/market/quote/:symbol", get(get_quote))
}

async fn get_quote(
    State(s): State<AppState>,
    Path(raw): Path<String>,
) -> Result<Json<Quote>, AppError> {
    let symbol = Symbol::parse(&raw).ok_or(AppError::BadSymbol)?;
    Ok(Json(s.market.quote(&symbol).await?))
}