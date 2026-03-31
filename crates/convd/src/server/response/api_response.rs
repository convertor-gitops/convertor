use crate::server::error::{AppError, AppStatus};
use crate::server::response::{RequestBody, collect_messages};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Display;

/// HTTP 200 成功响应的业务载体。
///
/// 只表达业务语义（成功或业务级失败），不持有 HTTP 状态码。
/// 最终由 `ResponseBody` 负责序列化和 `IntoResponse`。
#[derive(Default, Clone, Serialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: AppStatus,
    pub messages: Vec<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestBody>,
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            status: AppStatus::OK,
            messages: vec![],
            request: None,
            data: Some(data),
        }
    }

    pub fn business_error(error: AppError, request: RequestBody) -> ApiResponse<T> {
        Self {
            status: error.status,
            messages: collect_messages(&error),
            request: Some(request),
            data: None,
        }
    }

    pub fn set_status(mut self, status: AppStatus) -> Self {
        self.status = status;
        self
    }

    pub fn set_message(mut self, message: impl Display) -> Self {
        self.messages = vec![Cow::Owned(message.to_string())];
        self
    }

    pub fn set_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn set_request(mut self, request: RequestBody) -> Self {
        self.request = Some(request);
        self
    }
}

/// `ApiResponse` 直接作为 handler 返回值时（无错误路径），委托给 `ResponseBody`。
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        use crate::server::response::ResponseBody;
        ResponseBody::from(self).into_response()
    }
}
