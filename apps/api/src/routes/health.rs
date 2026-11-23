//! Health check route.
//!
//! A liveness endpoint for confirming the server is up and accepting
//! requests. Useful for local smoke tests, container health checks, and
//! load balancer probes.

use axum::{routing::get, Router};
use crate::state::AppState;

/// Builds the router for the health check.
///
/// The path is relative: the parent router nests this under `/api`, so the
/// full path is `/api/health`.
///
/// # `GET /api/health`
///
/// Always responds `200 OK` with the plain-text body `ok`. The response is
/// not JSON, unlike the rest of the API, so a probe can match on the body
/// without parsing.
///
/// # What it does not check
///
/// This is a **liveness** check only: it proves the process is running and
/// the router is responding. It does not call FMP, inspect the caches, or
/// validate `FMP_API_KEY`, so it will report `ok` even if the upstream API
/// is down or the key has been revoked. A readiness check that makes a
/// cheap upstream call would be a separate endpoint, and it should be
/// rate-limited or cached, since every call would count against your FMP
/// quota.
///
/// # Example
///
/// ```text
/// $ curl -i localhost:3001/api/health
/// HTTP/1.1 200 OK
/// content-type: text/plain; charset=utf-8
///
/// ok
/// ```
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(|| async { "ok" }))
}