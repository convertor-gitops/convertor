use crate::server::response::RequestBody;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Display;

/// HTTP 200 成功响应的业务载体。
///
/// 只表达业务语义（成功或业务级失败），不持有 HTTP 状态码。
/// 最终由 `ResponseBody` 负责序列化和 `IntoResponse`。
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: String,
    pub messages: Vec<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            messages: vec![],
            request: None,
            data: Some(data),
        }
    }

    pub fn set_message(mut self, message: impl Display) -> Self {
        self.messages = vec![Cow::Owned(message.to_string())];
        self
    }

    pub fn set_request(mut self, request: RequestBody) -> Self {
        self.request = Some(request);
        self
    }

    pub fn error(status: impl Display, error: impl core::error::Error) -> Self {
        let status = status.to_string();
        let mut messages = vec![Cow::Owned(error.to_string())];
        let mut source = error.source();
        while let Some(src) = source {
            messages.push(Cow::Owned(src.to_string()));
            source = src.source();
        }
        Self {
            status,
            messages,
            request: None,
            data: None::<T>,
        }
    }
}

/// `ApiResponse` 直接作为 handler 返回值时（无错误路径），委托给 `ResponseBody`。
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        use crate::server::response::ResponseBody;
        ResponseBody::from(self).into_response()
    }
}
