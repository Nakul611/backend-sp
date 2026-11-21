use crate::{
    error::AppError,
    models::{CreateItemRequest, Item},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_items(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let items = state
        .items
        .read()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();
    Ok(Json(items))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut items = state
        .items
        .write()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let next_id = items.iter().map(|i| i.id).max().unwrap_or(0) + 1;

    let new_item = Item {
        id: next_id,
        title: payload.title,
        description: payload.description,
        status: "Active".to_string(),
        created_at: now,
    };

    items.push(new_item.clone());
    Ok((StatusCode::CREATED, Json(new_item)))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<usize>,
) -> Result<impl IntoResponse, AppError> {
    let mut items = state
        .items
        .write()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let initial_len = items.len();
    items.retain(|i| i.id != id);

    if items.len() < initial_len {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Item with id {} not found", id)))
    }
}
