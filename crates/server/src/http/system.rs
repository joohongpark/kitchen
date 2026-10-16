use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Deserialize;
use serde_json::{Value, json};

use super::ApiResult;
use crate::auth::{SESSION_COOKIE, SESSION_DAYS};
use crate::state::AppState;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn info(State(s): State<AppState>) -> Json<Value> {
    Json(json!({
        "notifications": s.ntfy.is_some(),
        "notify_hour": s.config.notify_hour,
        "expiry_warn_days": s.config.expiry_warn_days,
    }))
}

#[derive(Deserialize)]
pub struct LoginInput {
    password: String,
}

pub async fn login(
    State(s): State<AppState>,
    jar: CookieJar,
    Json(input): Json<LoginInput>,
) -> ApiResult<(CookieJar, StatusCode)> {
    let token = s.auth.login(&input.password).await?;
    let cookie = Cookie::parse(format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        SESSION_DAYS * 24 * 3600
    ))
    .map_err(common::Error::internal)?;
    Ok((jar.add(cookie), StatusCode::NO_CONTENT))
}

pub async fn logout(State(s): State<AppState>, jar: CookieJar) -> ApiResult<(CookieJar, StatusCode)> {
    if let Some(c) = jar.get(SESSION_COOKIE) {
        s.auth.logout(c.value()).await?;
    }
    Ok((jar.remove(Cookie::build(SESSION_COOKIE).path("/")), StatusCode::NO_CONTENT))
}

/// 로그인 필요 여부와 현재 로그인 상태
pub async fn me(State(s): State<AppState>, jar: CookieJar) -> ApiResult<Json<Value>> {
    let authenticated = match (s.auth.enabled(), jar.get(SESSION_COOKIE)) {
        (false, _) => true,
        (true, Some(c)) => s.auth.verify(c.value()).await?,
        (true, None) => false,
    };
    Ok(Json(json!({ "auth_required": s.auth.enabled(), "authenticated": authenticated })))
}

pub async fn notify_test(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    if s.ntfy.is_none() {
        return Err(common::Error::invalid("NTFY_TOPIC 이 설정되지 않았습니다").into());
    }
    let id =
        s.queue.enqueue("notify", json!({ "title": "부엌 알림 테스트", "message": "알림이 잘 도착했어요 🍲" })).await?;
    Ok(Json(json!({ "job_id": id })))
}
