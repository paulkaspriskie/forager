use std::sync::Arc;
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use fmp_client::FmpError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid symbol")]
    BadSymbol,
    #[error("symbol not found")]
    NotFound,
    #[error("upstream data provider error")]
    Upstream,
}

impl From<Arc<FmpError>> for AppError {
    fn from(e: Arc<FmpError>) -> Self {
        match *e {
            FmpError::NotFound => AppError::NotFound,
            _ => {
                tracing::error!(error = %e, "fmp request failed");
                AppError::Upstream
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadSymbol => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Upstream => StatusCode::BAD_GATEWAY,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}