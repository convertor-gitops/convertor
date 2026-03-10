use crate::server::error::AppError;
use crate::server::response::{ApiError, ApiResponse, RequestBody};
use axum::http::StatusCode;
use convertor::error::{ConvQueryError, ConvUrlError, UrlBuilderError};
use serde::Serialize;

fn classify_build_url_input_error(error: &AppError) -> &'static str {
    match error {
        AppError::ConvQuery(ConvQueryError::UnsupportedClient(_, _))
        | AppError::UrlBuilder(UrlBuilderError::ConvQuery(ConvQueryError::UnsupportedClient(_, _))) => "UnsupportedClient",
        AppError::ConvQuery(ConvQueryError::MissingField(_, _))
        | AppError::UrlBuilder(UrlBuilderError::ConvQuery(ConvQueryError::MissingField(_, _))) => "MissingField",
        AppError::ConvQuery(ConvQueryError::Parse(_, _))
        | AppError::ConvQuery(ConvQueryError::Encode(_, _))
        | AppError::ConvQuery(ConvQueryError::Encrypt(_))
        | AppError::UrlBuilder(UrlBuilderError::ConvQuery(_)) => "InvalidQuery",
        AppError::UrlBuilder(UrlBuilderError::Url(_))
        | AppError::UrlBuilder(UrlBuilderError::NoSubHost(_))
        | AppError::UrlBuilder(UrlBuilderError::ConvUrlNoQuery)
        | AppError::UrlBuilder(UrlBuilderError::DownloadUrl(_, _))
        | AppError::ConvUrl(ConvUrlError::Url(_))
        | AppError::ConvUrl(ConvUrlError::NoPath(_))
        | AppError::ConvUrl(ConvUrlError::DeSerialQuery(_))
        | AppError::ConvUrl(ConvUrlError::Encrypt(_)) => "InvalidSubUrl",
        _ => "InvalidRequest",
    }
}

/// `build_url` 输入校验阶段错误 -> 业务错误（HTTP 200）。
pub fn map_build_url_input_error<T: Serialize>(error: AppError, request: RequestBody) -> ApiResponse<T> {
    let status = classify_build_url_input_error(&error);
    ApiResponse::error(status, error).set_request(request)
}

/// 依赖调用/内部执行阶段错误 -> HTTP 非200（当前沿用 error 自带 status）。
pub fn map_internal_error(error: AppError, request: RequestBody) -> ApiError {
    let status: StatusCode = error.status_code();
    ApiError::internal_server(status, error, request)
}
