use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use crate::services::config_service::ConfigService;
use serde_json::{json, Value};
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PageQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct FieldMutation {
    pub range_code: String,
    pub field: Value,
}

#[derive(Deserialize)]
pub struct DeleteFieldMutation {
    pub range_code: String,
    pub field_name: String,
}

pub fn config_routes(service: Arc<ConfigService>) -> Router {
    Router::new()
        .route("/api/pageconfig", get(get_config))
        .route("/api/pageconfig/:id/toconfig", post(sync_to_mongo))
        .route("/api/pageconfig/:id/fields", post(add_field).put(edit_field))
        .route("/api/pageconfig/:id/fields/delete", post(delete_field)) // Using post for delete if body is needed, or use proper delete and path/query
        .with_state(service)
}

async fn get_config(
    Query(query): Query<PageQuery>,
    State(service): State<Arc<ConfigService>>,
) -> impl IntoResponse {
    match service.get_config(&query.id).await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn sync_to_mongo(
    Path(id): Path<String>,
    State(service): State<Arc<ConfigService>>,
) -> impl IntoResponse {
    match service.sync_to_mongo(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Synced to MongoDB" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn add_field(
    Path(id): Path<String>,
    State(service): State<Arc<ConfigService>>,
    Json(payload): Json<FieldMutation>,
) -> impl IntoResponse {
    match service.add_field(&id, &payload.range_code, payload.field).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Field added" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn edit_field(
    Path(id): Path<String>,
    State(service): State<Arc<ConfigService>>,
    Json(payload): Json<FieldMutation>,
) -> impl IntoResponse {
    match service.edit_field(&id, &payload.range_code, payload.field).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Field updated" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn delete_field(
    Path(id): Path<String>,
    State(service): State<Arc<ConfigService>>,
    Json(payload): Json<DeleteFieldMutation>,
) -> impl IntoResponse {
    match service.delete_field(&id, &payload.range_code, &payload.field_name).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Field deleted" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
