//! API layer built with Axum 0.8.
//!
//! Routes:
//!   GET  /health                — liveness probe
//!   GET  /api/v1/feed           — personalised intelligence feed
//!   POST /api/v1/sources        — register a new source
//!   GET  /api/v1/sources        — list sources
//!   POST /api/v1/analyse        — on-demand analysis request
//!   GET  /api/v1/search         — full-text search

use axum::{routing::get, Router};

mod handlers;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/feed", get(handlers::feed))
        .route("/api/v1/sources", get(handlers::list_sources))
        .route("/api/v1/search", get(handlers::search))
}
