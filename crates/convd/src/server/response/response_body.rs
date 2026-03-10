use crate::server::response::{ApiError, ApiResponse, RequestBody};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::borrow::Cow;
use thiserror::__private17::AsDynError;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

/// 最终写入 HTTP Body 的统一 JSON 结构。
///
/// 无论成功还是失败，序列化后的 JSON 形状始终相同：
/// ```json
/// { "status": "ok"|"SomeError", "messages": [...], "request": {...}, "data": ... }
/// ```
///
/// - 成功路径由 `ApiResponse<T>` 转入，status = "ok"，data 有值
/// - 失败路径由 `ApiError` 转入，status = 错误 variant 名，data = null
#[derive(Serialize)]
pub struct ResponseBody<T: Serialize = ()> {
    pub status: String,
    pub messages: Vec<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 对应的 HTTP 状态码，序列化时跳过（仅用于 IntoResponse）
    #[serde(skip)]
    pub http_status: StatusCode,
}

impl<T: Serialize> From<ApiResponse<T>> for ResponseBody<T> {
    fn from(r: ApiResponse<T>) -> Self {
        Self {
            status: r.status,
            messages: r.messages,
            request: r.request,
            data: r.data,
            http_status: StatusCode::OK,
        }
    }
}

impl From<ApiError> for ResponseBody<()> {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Request { status, request, error } => Self {
                status: status.to_string(),
                messages: collect_messages(error.as_dyn_error()),
                request: Some(request),
                data: None,
                http_status: status,
            },
            ApiError::InternalServer { status, request, error } => Self {
                status: status.to_string(),
                messages: collect_messages(error.as_dyn_error()),
                request: Some(request),
                data: None,
                http_status: status,
            },
        }
    }
}

fn collect_messages(error: &(dyn std::error::Error + 'static)) -> Vec<Cow<'static, str>> {
    let mut messages = vec![Cow::Owned(error.to_string())];
    let mut err = error;

    while let Some(source) = err.source() {
        messages.push(Cow::Owned(source.to_string()));
        err = source;
    }

    messages
}

impl<T: Serialize> IntoResponse for ResponseBody<T> {
    fn into_response(self) -> Response {
        let http_status = self.http_status;
        let mut buf = BytesMut::with_capacity(256).writer();
        match serde_json::to_writer(&mut buf, &self) {
            Ok(()) => (
                http_status,
                [(header::CONTENT_TYPE, HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()))],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, mime::TEXT_PLAIN_UTF_8.as_ref())],
                Bytes::from(format!("Failed to serialize response: {}", err)),
            )
                .into_response(),
        }
    }
}

/// axum 拦截器：route 层返回 `Result<ApiResponse<T>, ApiError>`，
/// axum 调用 `HandlerResult<T>::from(result)` 后再 `into_response()`，route 层完全无感。
///
/// 用新类型包装绕开孤儿规则（`Result` 是标准库类型，无法直接 impl 外部 trait）。
pub struct HandlerResult<T: Serialize = ()>(pub Result<ApiResponse<T>, ApiError>);

impl<T: Serialize> From<Result<ApiResponse<T>, ApiError>> for HandlerResult<T> {
    fn from(result: Result<ApiResponse<T>, ApiError>) -> Self {
        HandlerResult(result)
    }
}

impl<T: Serialize> IntoResponse for HandlerResult<T> {
    fn into_response(self) -> Response {
        match self.0 {
            Ok(response) => ResponseBody::from(response).into_response(),
            Err(error) => ResponseBody::from(error).into_response(),
        }
    }
}
