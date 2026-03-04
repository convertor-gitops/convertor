pub mod subscription {
    use crate::server::app_state::AppState;
    use crate::server::response::{ApiError, ApiResponse, AppError, RequestSnapshot};
    use crate::server::router::{ConvertorQueryExtractor, OptionalScheme};
    use axum::body::Body;
    use axum::extract::{Path, State};
    use axum::http::Request;
    use axum_extra::extract::Host;
    use axum_extra::headers::HeaderMap;
    use convertor::config::proxy_client::ProxyClient;
    use convertor::config::subscription_config::Headers;
    use convertor::url::conv_query::ConvQuery;
    use convertor::url::url_builder::UrlBuilder;
    use convertor::url::url_result::UrlResult;
    use std::sync::Arc;

    #[tracing::instrument(skip_all)]
    pub async fn subscription(
        Path(client): Path<ProxyClient>,
        ConvertorQueryExtractor(query): ConvertorQueryExtractor,
        State(state): State<Arc<AppState>>,
        header_map: HeaderMap,
        Host(host): Host,
        OptionalScheme(scheme): OptionalScheme,
        request: Request<Body>,
    ) -> Result<ApiResponse<UrlResult>, ApiError> {
        let (parts, _) = request.into_parts();
        let request = RequestSnapshot::from_parts(scheme.unwrap_or("http".to_string()), host, parts);
        let response = internal_subscription(client, query, state, header_map).await;
        match response {
            Ok(response) => Ok(response.with_request(request)),
            Err(err) => Err(err.with_request(request)),
        }
        // 调试用
        // Ok(ApiResponse::ok(UrlResult::empty()))
    }

    #[tracing::instrument(skip_all)]
    async fn internal_subscription(
        client: ProxyClient,
        query: ConvQuery,
        state: Arc<AppState>,
        header_map: HeaderMap,
    ) -> Result<ApiResponse<UrlResult>, ApiError> {
        let query = query.check_for_subscription().map_err(ApiError::bad_request)?;
        let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client).map_err(ApiError::bad_request)?;
        let sub_url = url_builder.build_raw_url();
        let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers);
        let raw_profile = state
            .provider
            .get_raw_profile(sub_url.into(), headers)
            .await
            .map_err(ApiError::internal_server_error)?;
        let policies = match client {
            ProxyClient::Surge => {
                let mut profile = state
                    .surge_service
                    .try_get_profile(url_builder.clone(), raw_profile)
                    .await
                    .map_err(ApiError::internal_server_error)?;
                std::mem::take(&mut profile.sorted_policy_list)
            }
            ProxyClient::Clash => {
                let mut profile = state
                    .clash_service
                    .try_get_profile(url_builder.clone(), raw_profile)
                    .await
                    .map_err(ApiError::internal_server_error)?;
                std::mem::take(&mut profile.sorted_policy_list)
            }
        };
        let raw_url = url_builder.build_raw_url();
        let raw_profile_url = url_builder.build_raw_profile_url().map_err(ApiError::internal_server_error)?;
        let profile_url = url_builder.build_profile_url().map_err(ApiError::internal_server_error)?;
        let rule_providers_url = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy).map_err(AppError::UrlBuilderError))
            .collect::<Result<Vec<_>, AppError>>()
            .map_err(ApiError::internal_server_error)?;
        let url_result = UrlResult {
            raw_url,
            raw_profile_url,
            profile_url,
            rule_providers_url,
        };
        Ok(ApiResponse::ok(url_result))
    }
}
