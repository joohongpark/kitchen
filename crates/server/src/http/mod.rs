//! HTTP 계층: 요청을 서비스 입력으로, 서비스 결과를 응답으로 변환만 한다.

mod error;
mod ingredients;
mod pantry;
mod photos;
mod recipes;
mod shopping;
mod system;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum_extra::extract::CookieJar;
use tower_http::trace::TraceLayer;

pub use error::ApiError;

use crate::auth::SESSION_COOKIE;
use crate::state::AppState;
use crate::webapp;

pub type ApiResult<T> = Result<T, ApiError>;

const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/ingredients", get(ingredients::list).post(ingredients::create))
        .route("/ingredients/{id}", get(ingredients::get).put(ingredients::update).delete(ingredients::delete))
        .route("/pantry", get(pantry::list).post(pantry::create))
        .route("/pantry/adjust", post(pantry::adjust))
        .route("/pantry/expiring", get(pantry::expiring))
        .route("/pantry/{id}", get(pantry::get).put(pantry::update).delete(pantry::delete))
        .route("/recipes", get(recipes::list).post(recipes::create))
        .route("/recipes/recommend", post(recipes::recommend))
        .route("/recipes/{id}", get(recipes::get).put(recipes::update).delete(recipes::delete))
        .route("/recipes/{id}/cook-plan", get(recipes::cook_plan))
        .route("/recipes/{id}/cook", post(recipes::cook))
        .route("/recipes/{id}/cook-logs", get(recipes::cook_logs))
        .route("/shopping", get(shopping::list).post(shopping::create))
        .route("/shopping/clear-done", post(shopping::clear_done))
        .route("/shopping/{id}", axum::routing::put(shopping::update).delete(shopping::delete))
        .route("/photos", post(photos::upload))
        .route("/photos/{id}", get(photos::get))
        .route("/notifications/test", post(system::notify_test))
        .route("/info", get(system::info))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth))
        // 인증 없이 접근 가능
        .route("/health", get(system::health))
        .route("/auth/login", post(system::login))
        .route("/auth/logout", post(system::logout))
        .route("/auth/me", get(system::me))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES));

    Router::new().nest("/api", api).fallback(webapp::serve).layer(TraceLayer::new_for_http()).with_state(state)
}

async fn require_auth(State(state): State<AppState>, jar: CookieJar, req: Request, next: Next) -> Response {
    if !state.auth.enabled() {
        return next.run(req).await;
    }
    let ok = match jar.get(SESSION_COOKIE) {
        Some(c) => state.auth.verify(c.value()).await.unwrap_or(false),
        None => false,
    };
    if ok { next.run(req).await } else { ApiError(common::Error::Unauthorized).into_response() }
}
