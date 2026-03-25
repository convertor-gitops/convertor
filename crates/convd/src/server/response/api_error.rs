use crate::server::error::AppError;
use crate::server::response::RequestBody;
use axum::http;
use axum::response::{IntoResponse, Response};

/// HTTP 非200 失败响应的载体。
///
/// 最终由 `ResponseBody` 负责序列化和 `IntoResponse`。
#[derive(Debug)]
pub struct ApiError {
    pub error: AppError,
    pub request: RequestBody,
    pub http_status: http::StatusCode,
}

impl ApiError {
    pub fn bad_request(error: AppError, request: RequestBody) -> Self {
        Self {
            error,
            request,
            http_status: http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn internal_server(error: AppError, request: RequestBody) -> Self {
        Self {
            error,
            request,
            http_status: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// `ApiError` 需要实现 `IntoResponse` 以满足 axum extractor `Rejection` 约束。
/// 内部委托给 `ResponseBody`，保持序列化逻辑唯一。
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use crate::server::response::ResponseBody;
        ResponseBody::from(self).into_response()
    }
}
