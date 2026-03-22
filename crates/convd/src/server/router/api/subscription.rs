use super::{build_original_url, gen_url_builder, get_original_profile, into_api_error};
use crate::ext::Boxed;
use crate::server::app_state::AppState;
use crate::server::error::{AppError, InvalidQueryError, RequestError, UnknownError};
use crate::server::extractor::{HeaderExtra, RequestExtra};
use crate::server::model::UrlResult;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use convertor::config::proxy_client::ProxyClient;
use convertor::url::conv_query::ConvQuery;
use convertor::url::conv_url::UrlType;
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
            let url_builder = gen_url_builder(state.clone(), query, UrlType::Raw.path())?;
            let sub_url: url::Url = build_original_url(&url_builder, UrlType::Raw.path())?;
            let original_profile = get_original_profile(state.clone(), sub_url, &headers).await?;
            let content = match &url_builder.client {
                ProxyClient::Surge => state.surge_service.render_raw_profile(url_builder, original_profile).await,
                ProxyClient::Clash => Ok(original_profile),
            }
            .boxed_map_err(UnknownError::Service)
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
            let url_builder = gen_url_builder(state.clone(), query, UrlType::Profile.path())?;
            let sub_url: url::Url = build_original_url(&url_builder, UrlType::Profile.path())?;
            let raw_profile = get_original_profile(state.clone(), sub_url, &headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
                ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
            }
            .boxed_map_err(UnknownError::Service)
            .map_err(AppError::InternalServer)
        },
        request,
    )
    .await
}

#[instrument(skip_all)]
pub async fn proxy_provider(
    RequestExtra(request): RequestExtra,
    HeaderExtra(headers): HeaderExtra,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, ApiError> {
    into_api_error(
        async move {
            let proxy_provider_name = query
                .take_proxy_provider_name()
                .map_err(Box::new)
                .map_err(InvalidQueryError::ConvQuery)
                .map_err(|s| RequestError::InvalidQuery(UrlType::ProxyProvider.path().to_string(), s))
                .map_err(AppError::Request)?;
            let url_builder = gen_url_builder(state.clone(), query, UrlType::ProxyProvider.path())?;
            let sub_url: url::Url = build_original_url(&url_builder, UrlType::ProxyProvider.path())?;
            let raw_profile = get_original_profile(state.clone(), sub_url, &headers).await?;
            match url_builder.client {
                ProxyClient::Surge => {
                    return Err(AppError::Request(RequestError::UnsupportedClient(
                        UrlType::ProxyProvider.path().to_string(),
                        ProxyClient::Surge,
                    )));
                }
                ProxyClient::Clash => {
                    state
                        .clash_service
                        .proxy_provider(url_builder, raw_profile, proxy_provider_name)
                        .await
                }
            }
            .boxed_map_err(UnknownError::Service)
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
                .take_rule_provider_policy()
                .map_err(Box::new)
                .map_err(InvalidQueryError::ConvQuery)
                .map_err(|s| RequestError::InvalidQuery(UrlType::RuleProvider.path().to_string(), s))
                .map_err(AppError::Request)?;
            let url_builder = gen_url_builder(state.clone(), query, UrlType::RuleProvider.path())?;
            let sub_url: url::Url = build_original_url(&url_builder, UrlType::RuleProvider.path())?;
            let raw_profile = get_original_profile(state.clone(), sub_url, &headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.rule_provider(url_builder, raw_profile, &policy).await,
                ProxyClient::Clash => state.clash_service.rule_provider(url_builder, raw_profile, &policy).await,
            }
            .boxed_map_err(UnknownError::Service)
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
            let url_builder = gen_url_builder(state.clone(), query, "/api/build-url")?;
            let sub_url = build_original_url(&url_builder, "/api/build-url")?;

            // Phase 2: 依赖调用/内部执行（失败按 HTTP 错误返回）
            let raw_profile = get_original_profile(state.clone(), sub_url, &headers).await?;

            state
                .build_url_service
                .build_url(state.clone(), url_builder, raw_profile)
                .await
                .map(|r| ApiResponse::ok(r).set_request(cloned_request))
                .boxed_map_err(UnknownError::Service)
                .map_err(AppError::InternalServer)
        },
        request,
    )
    .await
}
