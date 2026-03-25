use crate::server::error::AppStatus;
use crate::server::response::{ApiError, ApiResponse, RequestBody, collect_messages};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::borrow::Cow;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

/// 最终写入 HTTP Body 的统一 JSON 结构。
///
/// 无论成功还是失败，序列化后的 JSON 形状始终相同：
///
/// - 成功路径由 `ApiResponse<T>` 转入
/// - 失败路径由 `ApiError` 转入
#[derive(Serialize)]
pub struct ResponseBody<T: Serialize = ()> {
    pub status: AppStatus,
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
        Self {
            status: e.error.status,
            messages: collect_messages(e.error),
            request: Some(e.request),
            data: None,
            http_status: e.http_status,
        }
    }
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
