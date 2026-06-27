use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::{HeaderExtractor, RequestExtractor};
use crate::server::model::UrlResult;
use crate::server::response::{ApiResponse, RequestBody};
use crate::server::router::helper::{build_original_url, gen_url_builder, get_original_profile};
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use color_eyre::eyre::WrapErr;
use convertor::url::conv_query::ConvQuery;
use serde::Serialize;
use serde_qs::axum::QsQuery;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/build-url", get(build_url))
        .route("/flush-cache", get(flush_cache))
        .route("/health", get(health))
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

#[tracing::instrument(skip_all)]
async fn health() -> ApiResponse<()> {
    ApiResponse::ok(())
}

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

#[tracing::instrument(skip_all)]
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
