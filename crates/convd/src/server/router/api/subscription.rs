use super::{build_original_url, gen_url_builder, get_original_profile, into_api_error};
use crate::server::app_state::AppState;
use crate::server::error::{AppError, InvalidQueryError, RequestError, UnknownError};
use crate::server::extractor::{HeaderExtra, RequestExtra};
use crate::server::model::UrlResult;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use convertor::config::proxy_client::ProxyClient;
use convertor::error::ConvQueryError;
use convertor::url::conv_query::ConvQuery;
use serde_qs::web::QsQuery;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn raw_profile(
    RequestExtra(request): RequestExtra,
    HeaderExtra(headers): HeaderExtra,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    into_api_error(
        async move {
            let url_builder = gen_url_builder(state.clone(), query)?;
            let sub_url: url::Url = build_original_url(&url_builder)?;
            let original_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            let content = match &url_builder.client {
                ProxyClient::Surge => state.surge_service.render_raw_profile(url_builder, original_profile).await,
                ProxyClient::Clash => Ok(original_profile),
            }
            .map_err(UnknownError::Service)
            .map_err(AppError::InternalServer)?;
            Ok(content)
        },
        request,
    )
    .await
}

#[instrument(skip_all)]
pub async fn profile(
    RequestExtra(request): RequestExtra,
    HeaderExtra(headers): HeaderExtra,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    into_api_error(
        async move {
            let url_builder = gen_url_builder(state.clone(), query)?;
            let sub_url: url::Url = build_original_url(&url_builder)?;
            let raw_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
                ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
            }
            .map_err(UnknownError::Service)
            .map_err(AppError::InternalServer)
        },
        request,
    )
    .await
}

#[instrument(skip_all)]
pub async fn rule_provider(
    RequestExtra(request): RequestExtra,
    HeaderExtra(headers): HeaderExtra,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    into_api_error(
        async move {
            let policy = query
                .policy
                .take()
                .ok_or(ConvQueryError::MissingField(
                    "/api/rule-provider".to_string(),
                    "RuleProviderPolicy".to_string(),
                ))
                .map_err(Box::new)
                .map_err(InvalidQueryError::ConvQuery)
                .map_err(Box::new)
                .map_err(RequestError::InvalidQuery)
                .map_err(AppError::Request)?;
            let url_builder = gen_url_builder(state.clone(), query)?;
            let sub_url: url::Url = build_original_url(&url_builder)?;
            let raw_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.rule_provider(url_builder, raw_profile, policy).await,
                ProxyClient::Clash => state.clash_service.rule_provider(url_builder, raw_profile, policy).await,
            }
            .map_err(UnknownError::Service)
            .map_err(AppError::InternalServer)
        },
        request,
    )
    .await
}

#[tracing::instrument(skip_all)]
pub async fn build_url(
    RequestExtra(request): RequestExtra,
    HeaderExtra(headers): HeaderExtra,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ApiResponse<UrlResult>, ApiError> {
    let cloned_request = request.clone();
    into_api_error(
        async move {
            // Phase 1: 输入解析/校验（失败按业务错误返回 HTTP 200）
            let url_builder = gen_url_builder(state.clone(), query)?;
            let sub_url = build_original_url(&url_builder)?;

            // Phase 2: 依赖调用/内部执行（失败按 HTTP 错误返回）
            let raw_profile = get_original_profile(state.clone(), sub_url, headers).await?;

            state
                .build_url_service
                .build_url(state.clone(), url_builder, raw_profile)
                .await
                .map(|r| ApiResponse::ok(r).set_request(cloned_request))
                .map_err(UnknownError::Service)
                .map_err(AppError::InternalServer)
        },
        request,
    )
    .await
}
