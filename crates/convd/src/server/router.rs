pub mod actuator;
pub mod angular;
pub mod api;
pub mod download;

use crate::server::AppState;
use crate::server::layer::trace::convd_trace_layer;
use crate::server::response::ApiError;
use axum::Router;
use axum::response::Redirect;
use axum::routing::{any, get};
use axum_prometheus::PrometheusMetricLayer;
use std::sync::Arc;

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
        .route("/api/raw", get(api::raw_profile))
        .route("/api/profile", get(api::profile))
        .route("/api/rule-provider", get(api::rule_provider))
        .route("/api/build-url", get(api::build_url))
        .route("/api/health", get(|| async { Ok::<_, ApiError>(()) }))
        .route("/download", any(download::download))
        .nest("/dashboard/", angular::router())
        .with_state(Arc::new(app_state))
        .layer(convd_trace_layer())
        .layer(prome_layer)
}

// pub struct ConvQueryExtractor(pub ConvQuery);
//
// impl FromRequestParts<Arc<AppState>> for ConvQueryExtractor {
//     type Rejection = ApiError;
//
//     async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
//         let scheme = OptionalScheme::from_request_parts(parts, state)
//             .await?
//             .0
//             .unwrap_or_else(|| "http".to_string());
//         let host = Host::from_request_parts(parts, state)
//             .await
//             .map_err(|e| ConvQueryError::NoHost(Box::new(e)))
//             .map_err(AppError::ConvQuery);
//         let request = RequestSnapshot::from_parts(
//             scheme.clone(),
//             host.as_ref().map(|h| h.0.as_str()).unwrap_or("").to_string(),
//             parts.clone(),
//         );
//         let query: Result<Self, Self::Rejection> = async {
//             let Host(host) = host.map_err(ApiError::bad_request)?;
//             let server = Url::parse(format!("{scheme}://{host}").as_str())
//                 .map_err(ConvQueryError::Url)
//                 .map_err(AppError::ConvQuery)
//                 .map_err(ApiError::bad_request)?;
//             let query_string = parts
//                 .uri
//                 .query()
//                 .ok_or(ConvQueryError::EmptyQuery)
//                 .map_err(AppError::ConvQuery)
//                 .map_err(ApiError::bad_request)?;
//             let query = ConvQuery::parse_from_query_string(query_string, &state.config.secret, server).map_err(ApiError::bad_request)?;
//             Ok(ConvQueryExtractor(query))
//         }
//         .await;
//         match query {
//             Ok(query) => Ok(query),
//             Err(err) => Err(err.with_request(request)),
//         }
//     }
// }
