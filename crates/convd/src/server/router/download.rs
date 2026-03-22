use crate::server::app_state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONNECTION, HOST, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION, TE, TRAILER, TRANSFER_ENCODING, UPGRADE};
use axum::http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use futures_util::TryStreamExt;
use reqwest::Url;
use serde::Deserialize;
use serde_qs::axum::QsQuery;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub url: String,
}

pub async fn download(State(state): State<Arc<AppState>>, QsQuery(q): QsQuery<DownloadQuery>, req: Request<Body>) -> Response<Body> {
    let url = match Url::parse(&q.url) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        Ok(_) => return text_response(StatusCode::BAD_REQUEST, "url scheme must be http or https"),
        Err(e) => return text_response(StatusCode::BAD_REQUEST, format!("invalid url: {e}")),
    };

    let client = state.download_client.clone();

    let (parts, body) = req.into_parts();

    let mut upstream_req = client.request(parts.method.clone(), url);

    // 透传请求头，但过滤 hop-by-hop / 不应透传的头
    for (name, value) in &parts.headers {
        if should_skip_request_header(name) {
            continue;
        }
        upstream_req = upstream_req.header(name, value);
    }

    // 流式透传请求体
    let req_stream = body.into_data_stream().map_err(std::io::Error::other);

    upstream_req = upstream_req.body(reqwest::Body::wrap_stream(req_stream));

    let upstream_resp = match upstream_req.send().await {
        Ok(resp) => resp,
        Err(e) => return text_response(StatusCode::BAD_GATEWAY, format!("upstream request failed: {e}")),
    };

    let status = upstream_resp.status();
    let mut resp_builder = Response::builder().status(status);

    // 透传响应头，但过滤 hop-by-hop / 不应透传的头
    for (name, value) in upstream_resp.headers() {
        if should_skip_response_header(name) {
            continue;
        }
        resp_builder = resp_builder.header(name, value);
    }

    // 流式透传响应体
    let resp_stream = upstream_resp.bytes_stream().map_err(std::io::Error::other);

    resp_builder
        .body(Body::from_stream(resp_stream))
        .unwrap_or_else(|e| text_response(StatusCode::INTERNAL_SERVER_ERROR, format!("failed to build response: {e}")))
}

fn should_skip_request_header(name: &HeaderName) -> bool {
    matches!(
        *name,
        CONNECTION | PROXY_AUTHENTICATE | PROXY_AUTHORIZATION | TE | TRAILER | TRANSFER_ENCODING | UPGRADE | HOST
    ) || name.as_str().eq_ignore_ascii_case("proxy-connection")
}

fn should_skip_response_header(name: &HeaderName) -> bool {
    matches!(
        *name,
        CONNECTION | PROXY_AUTHENTICATE | PROXY_AUTHORIZATION | TE | TRAILER | TRANSFER_ENCODING | UPGRADE
    ) || name.as_str().eq_ignore_ascii_case("proxy-connection")
}

fn text_response(status: StatusCode, msg: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )
        .body(Body::from(msg.into()))
        .unwrap()
}
