//! React 빌드 결과물(web/dist)을 바이너리에 내장해 제공한다. `/api` 외의 모든 경로는 SPA 로 보낸다.

use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web/dist"]
struct Assets;

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !path.is_empty()
        && let Some(file) = Assets::get(path)
    {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let cache = if path.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
        return (
            [
                (header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap()),
                (header::CACHE_CONTROL, HeaderValue::from_static(cache)),
            ],
            file.data,
        )
            .into_response();
    }
    match Assets::get("index.html") {
        Some(index) => ([(header::CACHE_CONTROL, "no-cache")], Html(index.data)).into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Html("<p>프론트엔드가 빌드되지 않았습니다. <code>cd web && npm run build</code> 후 다시 빌드하세요.</p>"),
        )
            .into_response(),
    }
}
