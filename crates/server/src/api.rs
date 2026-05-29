use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::services::ServeDir;
use tracker_core::{Dataset, Item, NewItem, Valuation};

use crate::store::Store;

type Shared = Arc<Mutex<Store>>;

/// Build the full router. `dist` optionally serves the WASM bundle as a fallback.
pub fn build_router(data_path: impl Into<PathBuf>, dist: Option<PathBuf>) -> Router {
    let state: Shared = Arc::new(Mutex::new(Store::new(data_path)));
    let api = Router::new()
        .route("/api/data", get(get_data))
        .route("/api/items", post(create_item))
        .route("/api/items/{id}", axum::routing::put(update_item).delete(delete_item))
        .route("/api/items/{id}/valuations", post(add_valuation))
        .with_state(state);

    match dist {
        Some(dir) => api.fallback_service(ServeDir::new(dir).append_index_html_on_directories(true)),
        None => api,
    }
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

fn lock(state: &Shared) -> std::sync::MutexGuard<'_, Store> {
    state.lock().expect("store mutex poisoned")
}

async fn get_data(State(state): State<Shared>) -> Result<Json<Dataset>, ApiError> {
    Ok(Json(lock(&state).load()?))
}

async fn create_item(
    State(state): State<Shared>,
    Json(payload): Json<NewItem>,
) -> Result<Json<Item>, ApiError> {
    validate_new(&payload)?;
    Ok(Json(lock(&state).add_item(payload)?))
}

async fn update_item(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(item): Json<Item>,
) -> Result<StatusCode, ApiError> {
    if item.name.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "name is required".into()));
    }
    match lock(&state).update_item(&id, item)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

async fn delete_item(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    match lock(&state).delete_item(&id)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

async fn add_valuation(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(valuation): Json<Valuation>,
) -> Result<StatusCode, ApiError> {
    if valuation.date.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "date is required".into()));
    }
    match lock(&state).add_valuation(&id, valuation)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

fn validate_new(p: &NewItem) -> Result<(), ApiError> {
    if p.name.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "name is required".into()));
    }
    if p.currency.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "currency is required".into()));
    }
    if p.purchase_price < 0.0 {
        return Err(ApiError(StatusCode::BAD_REQUEST, "purchasePrice must be >= 0".into()));
    }
    if p.purchase_date.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "purchaseDate is required".into()));
    }
    Ok(())
}
