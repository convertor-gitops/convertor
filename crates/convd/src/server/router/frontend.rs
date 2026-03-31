use crate::server::app_state::AppState;
use axum::Router;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_extra::headers::{HeaderMap, HeaderValue};
use include_dir::{Dir, include_dir};
use std::sync::Arc;
use tracing::instrument;

#[cfg(debug_assertions)]
static UI_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/development");
#[cfg(not(debug_assertions))]
static UI_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/production");

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // 注意：这里 / 与 /*path 都不做 3xx 重定向，直接 200 回应，避免重定向栈问题
        .route("/", get(index))
        .route("/{*path}", get(serve))
}

/// /dashboard/ 入口：直接回 index.html（非 3xx），交给 Angular 前端路由
#[instrument(skip_all)]
async fn index() -> Response {
    serve_index()
}

/// /dashboard/{*path}：先尝试当成静态文件命中；没命中则回 index.html（SPA fallback）
#[instrument(skip_all)]
async fn serve(Path(raw): Path<String>) -> Response {
    // 规范化与安全处理
    let path = raw.trim_start_matches('/');
    if path.contains("..") {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // 命中文件 → 直接回文件
    if let Some(file) = UI_DIST.get_file(path) {
        return respond_file(path, file.contents());
    }

    // 未命中 → 交给 Angular：直接回 index.html（非 3xx）
    serve_index()
}

/// 读取并返回内嵌的 index.html；并设置 no-cache，避免入口被缓存
fn serve_index() -> Response {
    match UI_DIST.get_file("index.html") {
        Some(index) => respond_index(index.contents()),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "index.html missing").into_response(),
    }
}

/// 通用：静态文件响应（带 Content-Type 与缓存策略）
fn respond_file(path: &str, bytes: &'static [u8]) -> Response {
    // 猜测 MIME（woff2、svg、js、css 等都会正确）
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    // index.html（理论上不会走到这里）不缓存；其余静态资源长期缓存（适配带 hash 的文件名）
    let cache_control = if std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("index.html"))
    {
        "no-cache, no-store, must-revalidate"
    } else {
        "public, max-age=31536000, immutable"
    };

    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_control));

    (headers, bytes).into_response()
}

/// 专门给 index.html：固定 text/html + no-cache
fn respond_index(bytes: &'static [u8]) -> Response {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    (headers, bytes).into_response()
}
