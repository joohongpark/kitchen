use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use common::Error;
use media::Photo;

use super::{ApiError, ApiResult};
use crate::state::AppState;

/// multipart 의 `file` 필드 하나를 받는다
pub async fn upload(State(s): State<AppState>, mut multipart: Multipart) -> ApiResult<(StatusCode, Json<Photo>)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| Error::invalid(e.to_string()))? {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| Error::invalid(e.to_string()))?;
            let photo = s.media.upload(bytes.to_vec()).await?;
            return Ok((StatusCode::CREATED, Json(photo)));
        }
    }
    Err(ApiError(Error::invalid("file 필드가 필요합니다")))
}

pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    let (photo, bytes) = s.media.read(&id).await?;
    Ok((
        [
            (header::CONTENT_TYPE, photo.mime),
            // 사진은 id 별로 바뀌지 않는다
            (header::CACHE_CONTROL, "private, max-age=31536000, immutable".to_string()),
        ],
        bytes,
    ))
}
