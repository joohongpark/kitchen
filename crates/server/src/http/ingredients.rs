use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use pantry::model::{Ingredient, IngredientInput, IngredientWithStock};

use super::ApiResult;
use crate::state::AppState;

pub async fn list(State(s): State<AppState>) -> ApiResult<Json<Vec<IngredientWithStock>>> {
    Ok(Json(s.pantry.list_ingredients().await?))
}

pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<IngredientWithStock>> {
    Ok(Json(s.pantry.get_ingredient(&id).await?))
}

pub async fn create(
    State(s): State<AppState>,
    Json(input): Json<IngredientInput>,
) -> ApiResult<(StatusCode, Json<Ingredient>)> {
    Ok((StatusCode::CREATED, Json(s.pantry.create_ingredient(input).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<IngredientInput>,
) -> ApiResult<Json<Ingredient>> {
    Ok(Json(s.pantry.update_ingredient(&id, input).await?))
}

pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    s.pantry.delete_ingredient(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
