use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::RequestExtractor;
use crate::server::model::{BackendStatus, ServiceStatus};
use crate::server::response::{ApiError, ApiResponse};
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use color_eyre::eyre::OptionExt;
use redis::AsyncTypedCommands;
use std::sync::Arc;
use tracing::instrument;

pub fn router(metrics_handle: PrometheusHandle) -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthy", get(healthy))
        .route("/ready", get(redis))
        .route("/redis", get(redis))
        .route("/status", get(status))
        .route(
            "/metrics",
            get(move || {
                let metrics_handle = metrics_handle.clone();
                async move { metrics_handle.render() }
            }),
        )
}

#[instrument(skip_all)]
async fn healthy() -> Result<ApiResponse<()>, ApiError> {
    Ok(ApiResponse::ok(()))
}

#[instrument(skip_all)]
async fn redis(RequestExtractor(request): RequestExtractor, State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let result: color_eyre::Result<_> = async move {
        let mut con = state.redis_connection.clone().ok_or_eyre("缺失 Redis 连接")?;
        let pone = con.ping().await?;
        Ok(ApiResponse::ok(pone))
    }
    .await;

    result
        .map_err(|r| AppError::new(AppStatus::NO_REDIS, r))
        .map_err(|e| ApiError::internal_server(e, request))
}

#[instrument(skip_all)]
async fn status(State(state): State<Arc<AppState>>) -> ApiResponse<BackendStatus> {
    let mut services = Vec::new();

    // Redis
    match state.redis_connection.clone() {
        Some(mut con) => match con.ping().await {
            Ok(_) => services.push(ServiceStatus::healthy("redis")),
            Err(e) => services.push(ServiceStatus::unhealthy("redis", e.to_string())),
        },
        None => services.push(ServiceStatus::unhealthy("redis", "未配置")),
    }

    // Loki
    match std::env::var("LOKI_URL") {
        Ok(url) if !url.is_empty() => services.push(ServiceStatus::healthy("loki")),
        _ => services.push(ServiceStatus::unhealthy("loki", "未配置 LOKI_URL")),
    }

    // Tempo (OTLP)
    match std::env::var("OTLP_GRPC") {
        Ok(url) if !url.is_empty() => services.push(ServiceStatus::healthy("tempo")),
        _ => services.push(ServiceStatus::unhealthy("tempo", "未配置 OTLP_GRPC")),
    }

    ApiResponse::ok(BackendStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    })
}
