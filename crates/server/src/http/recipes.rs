use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use recipes::model::*;
use serde::Deserialize;

use super::ApiResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    q: Option<String>,
    tag: Option<String>,
}

pub async fn list(State(s): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Json<Vec<RecipeSummary>>> {
    Ok(Json(s.recipes.list(q.q.as_deref(), q.tag.as_deref()).await?))
}

pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<RecipeDetail>> {
    Ok(Json(s.recipes.get(&id).await?))
}

pub async fn create(
    State(s): State<AppState>,
    Json(input): Json<RecipeInput>,
) -> ApiResult<(StatusCode, Json<RecipeDetail>)> {
    Ok((StatusCode::CREATED, Json(s.recipes.create(input).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<RecipeInput>,
) -> ApiResult<Json<RecipeDetail>> {
    Ok(Json(s.recipes.update(&id, input).await?))
}

pub async fn set_photo(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<SetPhotoInput>,
) -> ApiResult<Json<RecipeDetail>> {
    Ok(Json(s.recipes.set_photo(&id, input.photo_id.as_deref()).await?))
}

pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.recipes.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn recommend(
    State(s): State<AppState>,
    Json(req): Json<RecommendRequest>,
) -> ApiResult<Json<Vec<Recommendation>>> {
    Ok(Json(s.recipes.recommend(req).await?))
}

#[derive(Deserialize)]
pub struct PlanQuery {
    servings: Option<f64>,
}

pub async fn cook_plan(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<PlanQuery>,
) -> ApiResult<Json<CookPlan>> {
    Ok(Json(s.recipes.cook_plan(&id, q.servings).await?))
}

pub async fn cook(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CookRequest>,
) -> ApiResult<Json<CookResult>> {
    Ok(Json(s.recipes.cook(&id, req).await?))
}

pub async fn cook_logs(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<Vec<CookLog>>> {
    Ok(Json(s.recipes.cook_logs(&id).await?))
}
