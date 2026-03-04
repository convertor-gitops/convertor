pub mod actuator;
pub mod angular;
pub mod api;
pub mod profile;

use crate::server::AppState;
use crate::server::layer::trace::convd_trace_layer;
use crate::server::response::{ApiError, AppError, RequestSnapshot};
use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Redirect;
use axum::routing::get;
use axum_extra::extract::{Host, Scheme};
use axum_prometheus::PrometheusMetricLayer;
use convertor::error::QueryError;
use convertor::url::conv_query::ConvQuery;
use std::sync::Arc;
use url::Url;

pub fn router(app_state: AppState) -> Router {
    let (prome_layer, prome_handle) = PrometheusMetricLayer::pair();
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/dashboard", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/index.html", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/actuator/healthy", get(actuator::healthy))
        .route("/actuator/ready", get(actuator::redis))
        .route("/actuator/redis", get(actuator::redis))
        .route("/actuator/metrics", get(|| async move { prome_handle.render() }))
        .route("/raw-profile/{client}", get(profile::raw_profile))
        .route("/profile/{client}", get(profile::profile))
        .route("/rule-provider/{client}", get(profile::rule_provider))
        .route("/api/subscription/{client}", get(api::subscription::subscription))
        .route("/api/health", get(|| async { Ok::<_, ApiError>(()) }))
        .nest("/dashboard/", angular::router())
        .with_state(Arc::new(app_state))
        .layer(convd_trace_layer())
        .layer(prome_layer)
}

#[derive(Debug, Clone)]
pub struct OptionalScheme(pub Option<String>);

impl<S> FromRequestParts<S> for OptionalScheme
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let scheme = Scheme::from_request_parts(parts, state).await;
        match scheme {
            Ok(scheme) => Ok(OptionalScheme(Some(scheme.0))),
            Err(_) => Ok(OptionalScheme(None)),
        }
    }
}

pub struct ConvertorQueryExtractor(pub ConvQuery);

impl FromRequestParts<Arc<AppState>> for ConvertorQueryExtractor {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let scheme = OptionalScheme::from_request_parts(parts, state)
            .await?
            .0
            .unwrap_or_else(|| "http".to_string());
        let host = Host::from_request_parts(parts, state)
            .await
            .map_err(|e| QueryError::NoHost(Box::new(e)))
            .map_err(AppError::QueryError);
        let request = RequestSnapshot::from_parts(
            scheme.clone(),
            host.as_ref().map(|h| h.0.as_str()).unwrap_or("").to_string(),
            parts.clone(),
        );
        let query: Result<Self, Self::Rejection> = async {
            let Host(host) = host.map_err(ApiError::bad_request)?;
            let server = Url::parse(format!("{scheme}://{host}").as_str())
                .map_err(QueryError::Url)
                .map_err(AppError::QueryError)
                .map_err(ApiError::bad_request)?;
            let query_string = parts
                .uri
                .query()
                .ok_or(QueryError::EmptyQuery)
                .map_err(AppError::QueryError)
                .map_err(ApiError::bad_request)?;
            let query = ConvQuery::parse_from_query_string(query_string, &state.config.secret, server).map_err(ApiError::bad_request)?;
            Ok(ConvertorQueryExtractor(query))
        }
        .await;
        match query {
            Ok(query) => Ok(query),
            Err(err) => Err(err.with_request(request)),
        }
    }
}
