use axum::Router;
use crate::state::AppState;

mod fundamentals;
mod health;
mod market;

pub fn api() -> Router<AppState> {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(health::router())
            .merge(market::router())
            .merge(fundamentals::router()),
    )
}