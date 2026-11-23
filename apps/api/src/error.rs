//! Application error type and its mapping to HTTP responses.
//!
//! [`AppError`] is the one error type returned by route handlers and
//! services. It is deliberately coarse: clients get a status code and a
//! short, generic message, while the detailed cause (upstream status
//! codes, response bodies, network errors) goes to the server logs.

use std::sync::Arc;
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use fmp_client::FmpError;

/// Errors that can be returned to API clients.
///
/// Each variant maps to one HTTP status (see [`IntoResponse`] below) and
/// carries a fixed message from its `#[error(...)]` attribute. No variant
/// holds upstream details, so nothing about the data provider (URLs, plan
/// information, response bodies) can leak into a client response.
///
/// | Variant     | Status | Meaning                                    |
/// |-------------|--------|--------------------------------------------|
/// | `BadSymbol` | `400`  | Ticker failed validation                   |
/// | `NotFound`  | `404`  | Ticker is well-formed but FMP has no data  |
/// | `Upstream`  | `502`  | Any other failure talking to FMP           |
///
/// Handlers can use `?` on anything that converts into `AppError`, which
/// today means `Arc<FmpError>` (the error type moka hands back from its
/// cache lookups).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The ticker failed [`Symbol::parse`](crate::symbol::Symbol::parse).
    ///
    /// Raised in route handlers, before any cache or upstream access.
    #[error("invalid symbol")]
    BadSymbol,

    /// FMP has no data for a ticker that was otherwise well-formed.
    #[error("symbol not found")]
    NotFound,

    /// A failure on the FMP side or in reaching it.
    ///
    /// Covers network errors, timeouts, non-success statuses, and
    /// responses that couldn't be parsed. The client only sees the generic
    /// message; the specific cause is logged when the error is converted.
    #[error("upstream data provider error")]
    Upstream,
}

/// Converts a failed upstream call into an [`AppError`].
///
/// Takes an `Arc<FmpError>` because that is what moka's `try_get_with`
/// returns: when several requests wait on the same fetch, they all share
/// one error value.
///
/// - [`FmpError::NotFound`] becomes [`AppError::NotFound`] and is not
///   logged, since it is an ordinary outcome for a mistyped ticker.
/// - Every other variant becomes [`AppError::Upstream`] and is logged at
///   `error` level with the full cause. This log line is the place to look
///   when a client reports a generic `502`.
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

/// Renders the error as an HTTP response.
///
/// The body is always JSON of the form `{"error": "<message>"}`, with the
/// message taken from the variant's `#[error(...)]` text:
///
/// ```json
/// { "error": "symbol not found" }
/// ```
///
/// # Limitation
///
/// This only covers errors that handlers return themselves. Rejections
/// raised by axum's extractors before a handler runs, such as an invalid
/// `?limit=abc` query string, bypass `AppError` and come back as plain
/// text.
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