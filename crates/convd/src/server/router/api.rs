use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::{HeaderExtractor, RequestExtractor};
use crate::server::model::UrlResult;
use crate::server::openapi::{ConvQueryParams, EmptyApiResponseDoc, UrlResultApiResponseDoc};
use crate::server::response::{ApiResponse, RequestBody};
use crate::server::router::helper::{build_original_url, gen_url_builder, get_original_profile};
use axum::extract::State;
use color_eyre::eyre::WrapErr;
use convertor::url::conv_query::ConvQuery;
use serde::Serialize;
use serde_qs::axum::QsQuery;
use std::sync::Arc;
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(build_url))
        .routes(routes!(flush_cache))
        .routes(routes!(health))
}

async fn into_api_response<T, F, Fut>(request: RequestBody, f: F) -> ApiResponse<T>
where
    T: Serialize,
    Fut: Future<Output = Result<T, AppError>>,
    F: FnOnce(&RequestBody) -> Fut,
{
    match f(&request).await {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::business_error(e, request),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "服务可用", body = EmptyApiResponseDoc)
    ),
    tag = "api"
)]
#[tracing::instrument(skip_all)]
async fn health() -> ApiResponse<()> {
    ApiResponse::ok(())
}

#[utoipa::path(
    get,
    path = "/build-url",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "生成订阅相关 URL 成功，业务错误也会以内层 status 字段返回", body = UrlResultApiResponseDoc)
    ),
    tag = "api"
)]
#[tracing::instrument(skip_all)]
async fn build_url(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResponse<UrlResult> {
    into_api_response(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;

        let url_result = state
            .build_url_service
            .build_url(state.clone(), url_builder, original_profile)
            .await
            .map_err(|r| AppError::new(AppStatus::SERVICE, r))?;

        Ok(url_result)
    })
    .await
}

#[utoipa::path(
    get,
    path = "/flush-cache",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "清除内存中与 Redis 中(如果存在)关于 original_profile 的缓存", body = UrlResultApiResponseDoc)
    ),
    tag = "api"
)]
#[instrument(skip_all)]
async fn flush_cache(
    RequestExtractor(request): RequestExtractor,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResponse<()> {
    into_api_response(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        state
            .provider
            .flush_cache(sub_url)
            .await
            .wrap_err("无法清除缓存")
            .map_err(|r| AppError::new(AppStatus::SERVICE, r))?;
        Ok(())
    })
    .await
}
