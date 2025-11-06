//! HTTP API crate placeholder.

use axum::{routing::get, Router};
use serde::Serialize;
use tracing::info;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn router() -> Router {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler() -> axum::Json<HealthResponse> {
    info!("Received health check");
    axum::Json(HealthResponse { status: "ok" })
}
