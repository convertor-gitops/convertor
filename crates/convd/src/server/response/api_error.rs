use crate::server::error::UnknownError;
use crate::server::error::{AppError, RequestError};
use crate::server::response::RequestBody;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// HTTP 非200 错误的载体。
///
/// 持有 HTTP 状态码和业务错误，不负责序列化。
/// 最终由 `ResponseBody` 负责序列化和 `IntoResponse`。
#[derive(Debug)]
pub enum ApiError {
    Request {
        status: StatusCode,
        error: RequestError,
        request: RequestBody,
    },
    InternalServer {
        status: StatusCode,
        error: UnknownError,
        request: RequestBody,
    },
}

impl ApiError {
    pub fn bad_request(error: RequestError, request: RequestBody) -> Self {
        let status = StatusCode::BAD_REQUEST;
        Self::Request { status, error, request }
    }

    pub fn internal_server(status: StatusCode, error: UnknownError, request: RequestBody) -> Self {
        Self::InternalServer { status, error, request }
    }

    pub fn from_app_error(app_error: AppError, request_body: RequestBody) -> Self {
        match app_error {
            AppError::Request(e) => Self::bad_request(e, request_body),
            AppError::InternalServer(e) => Self::internal_server(e.status_code(), e, request_body),
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
