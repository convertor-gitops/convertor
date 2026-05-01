use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::{HeaderExtractor, RequestExtractor};
use crate::server::openapi::ConvQueryParams;
use crate::server::response::{RequestBody, SubscriptionError};
use crate::server::router::helper::{build_original_url, gen_url_builder, get_original_profile};
use axum::extract::State;
use color_eyre::eyre::eyre;
use convertor::config::proxy_client::ProxyClient;
use convertor::url::conv_query::ConvQuery;
use serde_qs::web::QsQuery;
use std::sync::Arc;
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(raw_profile))
        .routes(routes!(profile))
        .routes(routes!(proxy_provider))
        .routes(routes!(rule_provider))
}

pub async fn into_subscription_error<F, Fut>(request: RequestBody, f: F) -> Result<String, SubscriptionError>
where
    Fut: Future<Output = Result<String, AppError>>,
    F: FnOnce(&RequestBody) -> Fut,
{
    f(&request).await.map_err(|err| SubscriptionError::from_app_error(err, request))
}

#[utoipa::path(
    get,
    path = "/raw",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回转换前的订阅文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn raw_profile(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match &url_builder.client {
            ProxyClient::Surge => state.surge_service.render_raw_profile(url_builder, original_profile).await,
            ProxyClient::Clash => Ok(original_profile),
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

#[utoipa::path(
    get,
    path = "/profile",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回转换后的订阅文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn profile(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => state.surge_service.profile(url_builder, original_profile).await,
            ProxyClient::Clash => state.clash_service.profile(url_builder, original_profile).await,
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

#[utoipa::path(
    get,
    path = "/proxy-provider",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回 Clash Proxy Provider 文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn proxy_provider(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let proxy_provider_name = query
            .take_proxy_provider_name()
            .map_err(|e| eyre!(e))
            .map_err(|r| AppError::new(AppStatus::MISSING_PROXY_PROVIDER_NAME, r))?;
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => {
                return Err(AppError::new(
                    AppStatus::UNSUPPORTED_CLIENT,
                    eyre!("无法对 Surge 客户端返回相应的 ProxyProvider 配置"),
                ));
            }
            ProxyClient::Clash => {
                state
                    .clash_service
                    .proxy_provider(url_builder, original_profile, proxy_provider_name)
                    .await
            }
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

#[utoipa::path(
    get,
    path = "/rule-provider",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回 Rule Provider 文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn rule_provider(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let policy = query
            .take_rule_provider_policy()
            .map_err(|e| eyre!(e))
            .map_err(|r| AppError::new(AppStatus::MISSING_RULE_PROVIDER_POLICY, r))?;
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => state.surge_service.rule_provider(url_builder, original_profile, &policy).await,
            ProxyClient::Clash => state.clash_service.rule_provider(url_builder, original_profile, &policy).await,
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}
