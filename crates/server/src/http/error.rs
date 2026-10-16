use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use common::Error;
use serde_json::json;

/// 도메인 오류 → HTTP 응답
#[derive(Debug)]
pub struct ApiError(pub Error);

impl<E: Into<Error>> From<E> for ApiError {
    fn from(err: E) -> Self {
        ApiError(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::AlreadyExists(_) | Error::InUse(_) => StatusCode::CONFLICT,
            Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = match &self.0 {
            Error::Internal(detail) => {
                tracing::error!(%detail, "internal error");
                "서버 내부 오류가 발생했습니다".to_string()
            }
            other => other.to_string(),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
