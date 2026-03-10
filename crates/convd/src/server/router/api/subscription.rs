use super::{build_sub_url, gen_url_builder, get_original_profile, into_api_error};
use crate::server::app_state::AppState;
use crate::server::error::AppError;
use crate::server::extractor::{HeaderExtra, RequestExtra};
use crate::server::model::UrlResult;
use crate::server::response::{ApiError, ApiResponse, HandlerResult, map_build_url_input_error, map_internal_error};
use axum::extract::State;
use convertor::config::proxy_client::ProxyClient;
use convertor::error::ConvQueryError;
use convertor::url::conv_query::ConvQuery;
use serde_qs::web::QsQuery;
use std::sync::Arc;
use tracing::instrument;
use url::Url;

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
            let sub_url: Url = build_sub_url(&url_builder)?;
            let original_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            match &url_builder.client {
                ProxyClient::Surge => Ok(state.surge_service.render_raw_profile(url_builder, original_profile).await?),
                ProxyClient::Clash => Err(AppError::ConvQuery(ConvQueryError::UnsupportedClient(
                    "/api/raw_profile".to_string(),
                    ProxyClient::Clash,
                ))),
            }
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
            let sub_url: Url = build_sub_url(&url_builder)?;
            let raw_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
                ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
            }
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
            let policy = query.policy.take().ok_or(ConvQueryError::MissingField(
                "/api/rule-provider".to_string(),
                "RuleProviderPolicy".to_string(),
            ))?;
            let url_builder = gen_url_builder(state.clone(), query)?;
            let sub_url: Url = build_sub_url(&url_builder)?;
            let raw_profile = get_original_profile(state.clone(), sub_url, headers).await?;
            match url_builder.client {
                ProxyClient::Surge => state.surge_service.rule_provider(url_builder, raw_profile, policy).await,
                ProxyClient::Clash => state.clash_service.rule_provider(url_builder, raw_profile, policy).await,
            }
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
) -> HandlerResult<UrlResult> {
    // Phase 1: 输入解析/校验（失败按业务错误返回 HTTP 200）
    let url_builder = match gen_url_builder(state.clone(), query) {
        Ok(url_builder) => url_builder,
        Err(error) => return Ok(map_build_url_input_error(error, request)).into(),
    };
    let sub_url = match build_sub_url(&url_builder) {
        Ok(sub_url) => sub_url,
        Err(error) => return Ok(map_build_url_input_error(error, request)).into(),
    };

    // Phase 2: 依赖调用/内部执行（失败按 HTTP 错误返回）
    let raw_profile = match get_original_profile(state.clone(), sub_url, headers).await {
        Ok(raw_profile) => raw_profile,
        Err(error) => return Err(map_internal_error(error, request)).into(),
    };

    match state.build_url_service.build_url(state.clone(), url_builder, raw_profile).await {
        Ok(url_result) => Ok(ApiResponse::ok(url_result).set_request(request)).into(),
        Err(error) => Err(map_internal_error(error, request)).into(),
    }
}
