use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn feed() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "items": [] })))
}

pub async fn list_sources() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "sources": [] })))
}

pub async fn search() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "results": [] })))
}
