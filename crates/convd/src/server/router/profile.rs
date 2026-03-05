use crate::server::app_state::AppState;
use crate::server::response::{ApiError, AppError, RequestError};
use crate::server::router::ConvertorQueryExtractor;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use convertor::config::proxy_client::ProxyClient;
use convertor::config::subscription_config::Headers;
use convertor::url::url_builder::UrlBuilder;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn raw_profile(
    Path(client): Path<ProxyClient>,
    ConvertorQueryExtractor(query): ConvertorQueryExtractor,
    State(state): State<Arc<AppState>>,
    header_map: HeaderMap,
) -> Result<String, ApiError> {
    let query = query.check_for_profile().map_err(ApiError::bad_request)?;
    let url_builder = UrlBuilder::from_conv_query(query, &state.config.secret, client).map_err(ApiError::bad_request)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers);
    let raw_profile = state
        .provider
        .get_raw_profile(sub_url.into(), headers)
        .await
        .map_err(ApiError::internal_server_error)?;
    match client {
        ProxyClient::Surge => {
            let raw_profile = state
                .surge_service
                .raw_profile(url_builder, raw_profile)
                .await
                .map_err(ApiError::internal_server_error)?;
            Ok(raw_profile)
        }
        ProxyClient::Clash => Err(ApiError::bad_request(AppError::RequestError(RequestError::UnsupportedClient(
            client,
        )))),
    }
}

#[instrument(skip_all)]
pub async fn profile(
    Path(client): Path<ProxyClient>,
    ConvertorQueryExtractor(query): ConvertorQueryExtractor,
    State(state): State<Arc<AppState>>,
    header_map: HeaderMap,
) -> Result<String, ApiError> {
    let query = query.check_for_profile().map_err(ApiError::bad_request)?;
    let url_builder = UrlBuilder::from_conv_query(query, &state.config.secret, client).map_err(ApiError::bad_request)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers);
    let raw_profile = state
        .provider
        .get_raw_profile(sub_url.into(), headers)
        .await
        .map_err(ApiError::internal_server_error)?;
    let profile = match client {
        ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
        ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
    }
    .map_err(ApiError::internal_server_error)?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(
    Path(client): Path<ProxyClient>,
    ConvertorQueryExtractor(query): ConvertorQueryExtractor,
    State(state): State<Arc<AppState>>,
    header_map: HeaderMap,
) -> Result<String, ApiError> {
    let (query, policy) = query.check_for_rule_provider().map_err(ApiError::bad_request)?;
    let url_builder = UrlBuilder::from_conv_query(query, &state.config.secret, client).map_err(ApiError::bad_request)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers);
    let raw_profile = state
        .provider
        .get_raw_profile(sub_url.into(), headers)
        .await
        .map_err(ApiError::internal_server_error)?;
    let rules = match client {
        ProxyClient::Surge => state.surge_service.rule_provider(url_builder, raw_profile, policy).await,
        ProxyClient::Clash => state.clash_service.rule_provider(url_builder, raw_profile, policy).await,
    }
    .map_err(ApiError::internal_server_error)?;
    Ok(rules)
}
