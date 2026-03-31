use crate::server::error::AppError;
use crate::server::response::{RequestBody, collect_messages};
use axum::http;
use axum::http::header;
use axum::response::{IntoResponse, Response};

pub struct SubscriptionError {
    pub error: AppError,
    pub request: RequestBody,
    pub http_status: http::StatusCode,
}

impl SubscriptionError {
    pub fn from_app_error(error: AppError, request: RequestBody) -> Self {
        Self {
            error,
            request,
            http_status: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn bad_request(error: AppError, request: RequestBody) -> Self {
        Self {
            error,
            request,
            http_status: http::StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> Response {
        let body = collect_messages(self.error).join("\n");
        (self.http_status, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
    }
}
