use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use pantry::model::{ShoppingItem, ShoppingItemInput};
use serde_json::{Value, json};

use super::ApiResult;
use crate::state::AppState;

pub async fn list(State(s): State<AppState>) -> ApiResult<Json<Vec<ShoppingItem>>> {
    Ok(Json(s.pantry.list_shopping().await?))
}

pub async fn create(
    State(s): State<AppState>,
    Json(input): Json<ShoppingItemInput>,
) -> ApiResult<(StatusCode, Json<ShoppingItem>)> {
    Ok((StatusCode::CREATED, Json(s.pantry.add_shopping(input).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ShoppingItemInput>,
) -> ApiResult<Json<ShoppingItem>> {
    Ok(Json(s.pantry.update_shopping(&id, input).await?))
}

pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.pantry.delete_shopping(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear_done(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let removed = s.pantry.clear_done_shopping().await?;
    Ok(Json(json!({ "removed": removed })))
}
