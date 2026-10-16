use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use pantry::model::{PantryItem, PantryItemInput, StockAdjustment};
use serde::Deserialize;

use super::ApiResult;
use crate::state::AppState;

pub async fn list(State(s): State<AppState>) -> ApiResult<Json<Vec<PantryItem>>> {
    Ok(Json(s.pantry.list_items().await?))
}

pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<PantryItem>> {
    Ok(Json(s.pantry.get_item(&id).await?))
}

pub async fn create(
    State(s): State<AppState>,
    Json(input): Json<PantryItemInput>,
) -> ApiResult<(StatusCode, Json<PantryItem>)> {
    Ok((StatusCode::CREATED, Json(s.pantry.create_item(input).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<PantryItemInput>,
) -> ApiResult<Json<PantryItem>> {
    Ok(Json(s.pantry.update_item(&id, input).await?))
}

pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.pantry.delete_item(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn adjust(State(s): State<AppState>, Json(input): Json<Vec<StockAdjustment>>) -> ApiResult<StatusCode> {
    s.pantry.adjust(input).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ExpiringQuery {
    days: Option<i64>,
}

pub async fn expiring(State(s): State<AppState>, Query(q): Query<ExpiringQuery>) -> ApiResult<Json<Vec<PantryItem>>> {
    let today = Utc::now().with_timezone(&s.config.timezone).date_naive();
    let until = today + Duration::days(q.days.unwrap_or(s.config.expiry_warn_days));
    Ok(Json(s.pantry.list_expiring(until).await?))
}
